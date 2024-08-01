use reqwest::Client as ReqwestClient;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tracing::error;

#[derive(Serialize, Debug)]
struct BotLog {
    log: String,
}

async fn send_log(log: String) {
    let data = BotLog { log };

    let django_url = String::from("http://127.0.0.1:8000/api/bot-log/");
    let client = ReqwestClient::new();

    let response = client.post(&django_url).json(&data).send().await;

    match response {
        Ok(_) => {}
        Err(e) => {
            error!("Error during POST request: {:?}", e);
        }
    }
}

async fn capture_and_send_logs() {
    let mut cmd = TokioCommand::new("./target/release/yoitsus")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdout = cmd.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        send_log(line).await;
    }

    let status = cmd.wait().await.expect("Failed to wait on child");
    println!("Child process exited with status: {}", status);
}

#[tokio::main]
async fn main() {
    capture_and_send_logs().await;
}
