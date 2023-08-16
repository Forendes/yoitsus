use crate::commands::utils::to_time;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let _ = match queue.current() {
            Some(current) => current,
            None => {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.colour(0xf38ba8)
                                .title(":warning: Nothing is playing right now.")
                                .timestamp(Timestamp::now())
                        })
                    })
                    .await?;

                return Ok(());
            }
        };

        let mut desc = String::from("+ - + - + - + - + - + - + - + - + - +\n");
        let mut total_time = 0;
        for (i, song) in queue.current_queue().iter().enumerate() {
            desc.push_str(&format!(
                "{}. {} - {}\n",
                i + 1,
                song.metadata().title.clone().unwrap(),
                song.metadata()
                    .artist
                    .clone()
                    .unwrap_or_else(|| String::from("Unknown"))
            ));
            total_time += song.metadata().duration.unwrap().as_secs()
        }

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xffffff)
                        .title(":notes: - Queue - :notes:")
                        .fields(vec![
                            ("Queue length", format!("{}", queue.len()), true),
                            ("Total time", to_time(total_time), true),
                        ])
                        .description(desc)
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
