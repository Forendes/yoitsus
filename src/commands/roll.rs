use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use reqwest::Client as ReqwestClient;
use serde::Serialize;
use tracing::error;
use tracing::info;

#[derive(Serialize, Debug)]
struct BotData {
    message: String,
    user_id: String,
}

pub async fn send_data_to_django() -> Result<(), reqwest::Error> {
    info!("Creating data");
    let data = BotData {
        message: "Hello from botka".to_string(),
        user_id: "228332".to_string(),
    };

    let django_url = String::from("http://127.0.0.1:8000/api/bot-data/");
    let client = ReqwestClient::new();

    let response = client.post(&django_url).json(&data).send().await;

    match response {
        Ok(response) => {
            info!("POST request successful: {:?}", response);
        }
        Err(e) => {
            error!("Error during POST request: {:?}", e);
            return Err(e);
        }
    }

    Ok(())
}

#[command]
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    info!("send_data_to_django()");

    let a = send_data_to_django().await;
    info!("Result of sending data: {:?}", a);

    let max = match args.single::<i32>() {
        Ok(v) => v,
        Err(why) => {
            println!("Err rolling: {:?}", why);
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xf38ba8)
                            .title(":warning: Range must be a number")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;

            return Ok(());
        }
    };

    if max <= 1 {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xf38ba8)
                        .title(":warning: Number must be equal to or greater than 2.")
                        .timestamp(Timestamp::now())
                })
            })
            .await?;

        return Ok(());
    }

    let mut rng = StdRng::from_entropy();
    let result = rng.gen_range(1..=max);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0xffffff)
                    .title(format!("You rolled: {}", result))
                    .description(format!("Range: 1 - {}", max))
                    .timestamp(Timestamp::now())
            })
        })
        .await?;

    Ok(())
}
