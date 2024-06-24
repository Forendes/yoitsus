use regex::Regex;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use songbird::input::Restartable;
use tokio::process::Command;
use tracing::{error, info};

use crate::commands::utils::to_time;

#[command]
#[aliases(p)]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let url = match args.clone().single::<String>() {
        Ok(url) => url.clone(),
        Err(_) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xf38ba8)
                            .title(":warning: Use the command like this: play <url> or <song name>")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
            return Ok(());
        }
    };

    let search = args.clone();

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // A seperate !join is inconvenient, so bot joins with !play if not in voice channel
    if manager.get(guild_id).is_none() {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.colour(0xf38ba8)
                                .title(":warning: Join a voice channel first!")
                                .timestamp(Timestamp::now())
                        })
                    })
                    .await?;

                return Ok(());
            }
        };

        let manager = songbird::get(ctx)
            .await
            .expect("Songbird Voice client placed in at initialisation.")
            .clone();

        let (_, success) = manager.join(guild_id, connect_to).await;

        if let Err(_channel) = success {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xf38ba8)
                            .title(":warning: error joining channel.")
                            .description("Please ensure I have the correct permissions.")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
        }
    }
    if let Some(handler_lock) = manager.get(guild_id) {

        // Handle YT Music by redirecting to youtube.com equivalent
        if url.clone().starts_with("http") && url.contains("music.") {
            let _ = url.replace("music.", "");
        }

        // search on youtube for video with given name and pick first from search result
        if !url.clone().starts_with("http") {
            let mut handler = handler_lock.lock().await;
            let source = match songbird::input::ytdl_search(search.message()).await {
                Ok(source) => source,
                Err(why) => {
                    println!("Err starting source: {:?}", why);

                    msg.channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.colour(0xf38ba8)
                                    .title(":warning: Error adding song to playlist")
                                    .description("This could mean that one of the songs in the playlist is unavailable.")
                                    .timestamp(Timestamp::now())
                            })
                        })
                        .await?;
                    return Ok(());
                }
            };

            let song = handler.enqueue_source(source);
            let mut i = 0;
            for queued_song in handler.queue().current_queue() {
                if let Some(duration) = queued_song.metadata().duration {
                    i += duration.as_secs()
                } else {
                    i += 0;
                }
            }

            let playtime = to_time(i);
            let metadata = song.metadata();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xffffff)
                            .title(":notes: Song added to the queue!")
                            .thumbnail(metadata.thumbnail.clone().unwrap_or_else(|| String::from("https://images.unsplash.com/photo-1611162616475-46b635cb6868?ixlib=rb-4.0.3")))
                            .description(format!(
                                "{} - {}",
                                metadata.title.clone().unwrap(),
                                metadata.artist.clone().unwrap()
                            ))
                            .fields(vec![
                                ("Songs queued", format!("{}", handler.queue().len()), true),
                                ("Total playtime", playtime, true)
                            ])
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
        // handle playlist
        } else if url.contains("playlist") {
            let get_raw_list = Command::new("yt-dlp")
                .args(["-j", "--flat-playlist", &url])
                .output()
                .await;

            let raw_list = match get_raw_list {
                Ok(list) => String::from_utf8(list.stdout).unwrap(),
                Err(_) => String::from("Error!"),
            };

            let re = Regex::new(r#""url": "(https://www.youtube.com/watch\?v=[A-Za-z0-9]{11})""#)
                .unwrap();

            let urls: Vec<String> = re
                .captures_iter(&raw_list)
                .map(|cap| cap[1].to_string())
                .collect();

            let mut playtime = String::from("0");
            let mut errors = 0;

            let mut msg = msg
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xffffff)
                            .title(":notes: Queuing tracks...")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;

            for url in urls {
                info!("Queueing --> {}", url);
                let source = match Restartable::ytdl(url, true).await {
                    Ok(source) => source,
                    Err(why) => {
                        error!("Err starting source: {:?}", why);
                        errors += 1;
                        continue;
                    }
                };
                let mut handler = handler_lock.lock().await;
                let _song = handler.enqueue_source(source.into());
                let mut i = 0;

                msg.edit(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xffffff)
                            .title(":notes: Queuing tracks...")
                            .fields(vec![(
                                "Songs queued",
                                format!("{}", handler.queue().len()),
                                true,
                            )])
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;

                for queued_song in handler.queue().current_queue() {
                    i += queued_song.metadata().duration.unwrap().as_secs();
                }

                playtime = to_time(i);
            }
            let handler = handler_lock.lock().await;
            msg.edit(&ctx.http, |m| {
                m.embed(|e| {
                    e.colour(0xffffff)
                        .title(":notes: Queued playlist!")
                        .fields(vec![
                            ("Songs queued", format!("{}", handler.queue().len()), true),
                            ("Total playtime", playtime, true),
                        ])
                        .timestamp(Timestamp::now())
                })
            })
            .await?;

            if errors >= 1 {
                let er = match errors {
                    1 => format!(":warning: Error adding {errors} song to playlist"),
                    _ => format!(":warning: Error adding {errors} songs to playlist"),
                };

                msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xf38ba8)
                            .title(er)
                            .description("This could mean that one of the songs in the playlist is unavailable.")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
            }
        // handle live stream
        } else if url.contains("live") {
            let mut handler = handler_lock.lock().await;
            let source = match Restartable::ytdl(url, true).await {
                Ok(source) => source,
                Err(why) => {
                    error!("Err starting source: {:?}", why);

                    msg.channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.colour(0xf38ba8)
                                    .title(":warning: Error adding song to playlist.")
                                    .description("This could mean that the song is unavailable.")
                                    .timestamp(Timestamp::now())
                            })
                        })
                        .await?;
                    return Ok(());
                }
            };

            let song = handler.enqueue_source(source.into());
            let metadata = song.metadata();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xffffff)
                            .title(":notes: Added to playlist!")
                            .thumbnail(metadata.thumbnail.clone().unwrap_or_else(|| String::from("https://images.unsplash.com/photo-1611162616475-46b635cb6868?ixlib=rb-4.0.3")))
                            .description(format!(
                                "{} - {}",
                                metadata.title.clone().unwrap(),
                                metadata.artist.clone().unwrap()
                            ))
                            .fields(vec![
                                ("Songs queued", format!("{}", handler.queue().len()), true),
                                ("Total playtime", "infinite".to_string(), true)
                            ])
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
        // handle direct link to a video
        } else {
            let source = match Restartable::ytdl(url, true).await {
                Ok(source) => source,
                Err(why) => {
                    error!("Err starting source: {:?}", why);

                    msg.channel_id
                        .send_message(&ctx.http, |m| {
                            m.embed(|e| {
                                e.colour(0xf38ba8)
                                    .title(":warning: Error adding song to playlist.")
                                    .description("This could mean that the song is unavailable.")
                                    .timestamp(Timestamp::now())
                            })
                        })
                        .await?;
                    return Ok(());
                }
            };
            let mut handler = handler_lock.lock().await;
            let song = handler.enqueue_source(source.into());
            let mut i = 0;
            for queued_song in handler.queue().current_queue() {
                i += queued_song.metadata().duration.unwrap().as_secs();
            }
            let playtime = to_time(i);
            let metadata = song.metadata();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xffffff)
                            .title(":notes: Added to playlist!")
                            .thumbnail(metadata.thumbnail.clone().unwrap_or_else(|| String::from("https://images.unsplash.com/photo-1611162616475-46b635cb6868?ixlib=rb-4.0.3")))
                            .description(format!(
                                "{} - {}",
                                metadata.title.clone().unwrap(),
                                metadata.artist.clone().unwrap()
                            ))
                            .fields(vec![
                                ("Songs queued", format!("{}", handler.queue().len()), true),
                                ("Total playtime", playtime, true)
                            ])
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
        }
    }
    Ok(())
}
