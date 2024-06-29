use chat_gpt_lib_rs::{ChatGPTClient, ChatInput, Message, Model, Role};
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::env;
use tracing::info;
use tracing::log::error;

#[command]
async fn askgpt(ctx: &Context, msg: &serenity::model::channel::Message) -> CommandResult {
    if msg.author.bot {
        info!("ignored bot message");
        return Ok(());
    }
    let mut content = msg.content.clone();
    content.replace_range(0..10, "");

    let api_key = env::var("GPT_API_KEY").expect("Set your GPT_API_KEY environment variable!");
    let base_url = "https://api.openai.com";
    let client = ChatGPTClient::new(&api_key, base_url);

    let chat_input = ChatInput {
        model: Model::Gpt3_5Turbo,
        messages: vec![
            Message {
                role: Role::System,
                content: "You're a helpful assistant".to_string(),
            },
            Message {
                role: Role::User,
                content: content.clone(),
            },
        ],
        ..Default::default()
    };

    let mut message = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0xffffff)
                    .title(":hourglass: Generating answer...".to_string())
                    .timestamp(Timestamp::now())
            })
        })
        .await?;

    let response = match client.chat(chat_input).await {
        Ok(response) => response,
        Err(why) => {
            error!("Err getting response from GPT: {:?}", why);
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xf38ba8)
                            .title(":warning: Unable to get response from ChatGPT.")
                            .timestamp(Timestamp::now())
                    })
                })
                .await?;
            return Ok(());
        }
    };

    let answer = &response.choices[0].message.content;
    message
        .edit(&ctx.http, |m| {
            m.embed(|e| {
                e.colour(0xffffff)
                    .title(format!("Question: {}", content))
                    .description(answer.to_string())
                    .timestamp(Timestamp::now())
            })
        })
        .await?;

    Ok(())
}
