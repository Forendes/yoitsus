mod commands;
mod web;

use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::time::{self, sleep, Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::standard::macros::{group, hook};
use serenity::framework::standard::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::event::ResumedEvent;
use serenity::model::gateway::Ready;
use serenity::model::prelude::Activity;
use serenity::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, instrument, Level};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{self, time::FormatTime};

use songbird::SerenityInit;

use crate::web::monitoring::*;

/* Import commands */
use crate::commands::help::*;
use crate::commands::roll::*;

use crate::commands::music::clear::*;
use crate::commands::music::join::*;
use crate::commands::music::leave::*;
use crate::commands::music::nowplaying::*;
use crate::commands::music::pause::*;
use crate::commands::music::play::*;
use crate::commands::music::queue::*;
use crate::commands::music::resume::*;
use crate::commands::music::shuffle::*;
use crate::commands::music::skip::*;
use crate::commands::music::stop::*;

/* Shards container */
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

static COMMAND_COUNT: AtomicUsize = AtomicUsize::new(0);
static START_TIME: AtomicUsize = AtomicUsize::new(0);

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "Connected as --> {} [id: {}]",
            ready.user.name, ready.user.id
        );
        let status =
            env::var("DISCORD_STATUS").expect("Set your DISCORD_STATUS environment variable!");
        ctx.set_activity(Activity::playing(status)).await;

        START_TIME.store(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize, Ordering::SeqCst);

        tokio::spawn(monitor_resources(ctx.clone()));
    }

    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        debug!("Resumed; trace: {:?}", resume.trace);
    }
}

#[hook]
#[instrument]
async fn before(_: &Context, msg: &Message, command_name: &str) -> bool {
    info!(
        "Received command --> '{}' || User --> '{}'",
        command_name, msg.author.name
    );
    COMMAND_COUNT.fetch_add(1, Ordering::SeqCst);
    true
}

async fn monitor_resources(ctx: Context) {
    let mut system =
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    let mut interval = time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        sleep(Duration::from_secs(
            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_secs(),
        ))
        .await;

        system.refresh_cpu();
        system.refresh_memory();

        let memory_usage_gb = system.used_memory() as f64 / 1_048_576.0;
        let cpu_usages: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let total_cpu_usage = cpu_usages.iter().sum::<f32>() / cpu_usages.len() as f32;

        // If CPU usage = 0 skip
        if total_cpu_usage == 0.0 {
            continue;
        }

        let memory_usage_gb = memory_usage_gb.to_string();
        let total_cpu_usage = total_cpu_usage.to_string();

        let start = Instant::now();
        ctx.http.get_gateway().await.ok();
        let elapsed = start.elapsed().as_millis().to_string();

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
        let uptime_seconds = current_time - START_TIME.load(Ordering::SeqCst);
        let uptime = format!("{}:{:02}:{:02}", uptime_seconds / 3600, (uptime_seconds / 60) % 60, uptime_seconds % 60);
        let command_count = COMMAND_COUNT.load(Ordering::SeqCst);

        send_data(memory_usage_gb, total_cpu_usage, elapsed, uptime, command_count.to_string()).await;

        let current_command_count = COMMAND_COUNT.load(Ordering::SeqCst);
        if current_command_count > command_count {
            COMMAND_COUNT.store(current_command_count - command_count, Ordering::SeqCst);
        } else {
            COMMAND_COUNT.store(0, Ordering::SeqCst);

        };
    }
}

#[derive(Debug)]
struct CustomTimeFormatter;

impl FormatTime for CustomTimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).unwrap();
        let seconds = duration.as_secs();

        let hours = (seconds / 3600) % 24;
        let minutes = (seconds / 60) % 60;
        let seconds = seconds % 60;

        write!(w, "[{:02}:{:02}:{:02}]", hours, minutes, seconds)
    }
}

#[group]
#[commands(
    // Misc
    help,   roll,  

    // Music commands
    leave,  play,   pause,  resume,  clear,
    skip,   stop,   queue,  shuffle, nowplaying,
    join,
)]
struct General;

#[tokio::main]
async fn main() {
    let format = fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false) 
        .with_thread_ids(false) 
        .with_thread_names(false) 
        .with_ansi(false)
        .compact(); 

    tracing_subscriber::fmt()
        .event_format(format)
        .with_timer(CustomTimeFormatter) 
        .with_max_level(Level::INFO) 
        .init();

    dotenvy::dotenv().expect("Failed to load .env file.");

    let token = env::var("DISCORD_TOKEN").expect("Set your DISCORD_TOKEN environment variable!");
    let prefix = env::var("PREFIX").expect("Set your PREFIX environment variable!");

    let http = Http::new(&token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.owners(owners).prefix(prefix))
        .before(before)
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .register_songbird()
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
