use tokio_tungstenite::tungstenite::protocol::Message;
use serde::Serialize;
use tokio_tungstenite::connect_async;
use tracing::error;
use futures_util::SinkExt;

#[derive(Serialize, Debug)]
pub struct BotResources {
    pub cpu_usage: String,
    pub memory_usage: String,
    pub latency: String,
    pub uptime: String,
    pub command_count: String,
}

pub async fn send_data(memory_usage: String, cpu_usage: String, latency: String, uptime: String, command_count: String) {
    let data = BotResources {
        cpu_usage,
        memory_usage,
        latency,
        uptime,
        command_count,
    };

    let websocket_url = "ws://127.0.0.1:8000/ws/bot-resources/";

    match connect_async(websocket_url).await {
        Ok((mut socket, _)) => {
            let json_data = match serde_json::to_string(&data) {
                Ok(json) => json,
                Err(e) => {
                    error!("Error serializing data: {:?}", e);
                    return;
                }
            };

            if let Err(e) = socket.send(Message::Text(json_data)).await {
                error!("Error sending message: {:?}", e);
            }
        }
        Err(e) => {
            error!("Error connecting to websocket: {:?}", e);
        }
    }
}
