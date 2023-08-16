use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::commands::utils::to_time;

#[command]
#[aliases("np")]
#[only_in(guilds)]
async fn nowplaying(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let current = match queue.current() {
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

        let metadata = current.metadata();
        let track_info = current.get_info().await.unwrap();

        let date_formatted = match &metadata.date {
            Some(date) => {
                format!("{}/{}/{}", &date[6..8], &date[4..6], &date[0..4])
            }
            None => String::from("Unknown"),
        };

        let time_formatted = {
            format!(
                "{} - {}",
                to_time(track_info.position.as_secs()),
                to_time(metadata.duration.unwrap().as_secs())
            )
        };

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e
                .colour(0xffffff)
                .title(metadata.title.clone().unwrap_or_else(|| String::from("Unknown")))
                .thumbnail(metadata.thumbnail.clone().unwrap_or_else(|| String::from("https://images.unsplash.com/photo-1611162616475-46b635cb6868?ixlib=rb-4.0.3")))
                .url(metadata.source_url.clone().unwrap())
                .fields(vec![
                    ("Artist", metadata.artist.clone().unwrap_or_else(|| String::from("Unknown")), false),
                    ("Released", date_formatted, true),
                    ("Position", time_formatted, true),
                    ("Status", format!("{:?}", track_info.playing), true),
                ])
                .timestamp(Timestamp::now())
            )
        }).await?;
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
