use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[only_in(guilds)]
// Joins voice channel, mostly not needed because !play tries to join vc, used in case
// if bot got kicked/disconnected from voice channel and !play can't join again.
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let bot_id = UserId::from(962051631619407963 as u64);

    let bot_joined = guild
        .voice_states
        .get(&bot_id)
        .and_then(|voice_state| voice_state.channel_id);

    // Check if bot in vc
    if manager.get(guild_id).is_none() | manager.get(guild_id).is_some() && bot_joined.is_none() {
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
            return Ok(());
        }
    }
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0xffffff)
                    .title("Joined voice channel!")
                    .timestamp(Timestamp::now())
            })
        })
        .await?;
    Ok(())
}
