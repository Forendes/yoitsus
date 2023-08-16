use rand::Rng;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[only_in(guilds)]
async fn shuffle(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        queue.modify_queue(|queue| {
            // skip the first track on queue because it's being played
            fisher_yates_shuffle(
                queue.make_contiguous()[1..].as_mut(),
                &mut rand::thread_rng(),
            )
        });

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xffffff)
                        .title(":notes: Queue shuffled!")
                        .timestamp(Timestamp::now())
                })
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xf38ba8)
                        .title(":warning: Not in a voice channel.")
                        .timestamp(Timestamp::now())
                })
            })
            .await?;
    }
    Ok(())
}

fn fisher_yates_shuffle<T, R>(arr: &mut [T], mut rng: R)
where
    R: rand::RngCore + Sized,
{
    let mut index = arr.len();
    while index >= 2 {
        index -= 1;
        arr.swap(index, rng.gen_range(0..(index + 1)));
    }
}
