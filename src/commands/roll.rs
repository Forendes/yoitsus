use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

#[command]
async fn roll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    let result = rng.gen_range(1..max);

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
