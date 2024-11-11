use std::collections::HashMap;

use clap::{Parser, Subcommand};

use reqwest::blocking::Client;

use serde_json::{Value, json};

use confy;

use serde::{Serialize, Deserialize};

use termimad::print_inline;

#[derive(Serialize, Deserialize)]
struct Config {
    slack_token: String,
    openai_token: String,
    request_url: String,
    model: String,
    max_tokens: i32,
    num_messages: i32,
}

impl ::std::default::Default for Config {
    fn default() -> Self { Self { slack_token: "xoxb-1234567890-abcdefghijklm-1234567890abcdefghijkl".into(), openai_token: "SOME_OPENAI_TOKEN".into(), request_url: "https://api.openai.com/v1/chat/completions".into(), model: "gpt-4o-mini".into(), max_tokens: 1000, num_messages: 20, } }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set the Slack bot token
    #[arg(short, long)]
    slack_token: Option<String>,

    /// Set the OpenAI token
    #[arg(short, long)]
    openai_token: Option<String>,

    /// Set the OpenAI request URL
    #[arg(short, long)]
    request_url: Option<String>,

    /// Set the OpenAI model
    #[arg(short, long)]
    model: Option<String>,

    /// Set the maximum number of output tokens
    #[arg(short, long)]
    tokens: Option<i32>,

    /// Set the number of messages to summarize
    #[arg(short, long)]
    num_messages: Option<i32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Summarize the last 20 messages sent in a Slack channel
    Summarize {
        /// The name of the channel to summarize
        channel: String,
    },
}

fn main() {
    let args = Args::parse();

    let cfg: Config = confy::load("slack-summarizer", None).expect("Failed to load configuration");

    let slack_token = args.slack_token.unwrap_or_else(|| {
        if cfg.slack_token == Config::default().slack_token {
            panic!("Slack token not found. Please provide a Slack token using the --slack-token flag. You only need to do this once.");
        }
        cfg.slack_token.clone()
    });

    let openai_token = args.openai_token.unwrap_or_else(|| {
        if cfg.openai_token == Config::default().openai_token {
            panic!("OpenAI token not found. Please provide an OpenAI token using the --openai-token flag. You only need to do this once.");
        }
        cfg.openai_token.clone()
    });

    let request_url = args.request_url.unwrap_or(cfg.request_url);
    let model = args.model.unwrap_or(cfg.model);
    let max_tokens = args.tokens.unwrap_or(cfg.max_tokens);
    let num_messages = args.num_messages.unwrap_or(cfg.num_messages);

    let channels = get_channels(slack_token.clone());
    let messages = if let Some(Commands::Summarize { channel }) = args.command {
        get_messages(slack_token.clone(), channels.get(&channel).unwrap().to_string(), num_messages.clone())
    } else {
        Vec::new()
    };

    let summary = summarize_messages(messages, openai_token.clone(), request_url.clone(), model.clone(), max_tokens.clone());
    let cleaned_summary = summary.replace("\\n", "\n").replace("\\", "") + "\n";
    print_inline(cleaned_summary.as_str());

    confy::store("slack-summarizer", None, Config { slack_token:slack_token, openai_token:openai_token, request_url:request_url, model:model, max_tokens:max_tokens, num_messages:num_messages }).expect("Failed to store configuration");

}

fn get_channels(bot_token: String) -> HashMap<String, String> {
    let client = Client::new();
    let response = client
        .get("https://slack.com/api/conversations.list")
        .header("Authorization", format!("Bearer {}", bot_token))
        .send().expect("Failed to send request");

    let response_json: Value = response.json().expect("Failed to parse JSON");

    let mut channels = HashMap::new();

    for channel in response_json["channels"].as_array().unwrap() {
        channels.insert(
            channel["name"].as_str().unwrap().to_string(),
            channel["id"].as_str().unwrap().to_string(),
        );
    }

    channels
}

fn get_messages(bot_token: String, channel_id: String, num_messages: i32) -> Vec<String> {
    join_channel(channel_id.clone(), bot_token.clone());

    let client = Client::new();
    let response = client
        .get("https://slack.com/api/conversations.history")
        .header("Authorization", format!("Bearer {}", bot_token))
        .query(&[("channel", channel_id), ("limit", num_messages.to_string())])
        .send().expect("Failed to send request");

    let response_json: Value = response.json().expect("Failed to parse JSON");

    let mut messages = Vec::new();

    for message in response_json["messages"].as_array().unwrap_or(&Vec::new()) {
        messages.push(message["text"].to_string());
    }

    messages
}

fn summarize_messages(messages: Vec<String>, openai_token: String, request_url: String, model: String, max_tokens: i32) -> String {
    let client = Client::new();
    let response = client
        .post(request_url)
        .header("Authorization", format!("Bearer {}", openai_token))
        .json(&json!({
            "messages": [
                {
                    "role": "system",
                    "content": "You are a Slack summarizer bot. You must summarize the most recent messages sent in a Slack channel. The following are the messages:"
                },
                {
                    "role": "user",
                    "content": messages.join("\n")
                }
            ],
            "temperature": 0.75,
            "top_p": 0.75,
            "max_tokens": max_tokens,
            "model": model
        }))
        .send().expect("Failed to send request");

    let response_json: Value = response.json().expect("Failed to parse JSON");

    response_json["choices"][0]["message"]["content"].to_string()
}

fn join_channel(channel_id: String, bot_token: String) {
    let client = Client::new();
    client
        .post("https://slack.com/api/conversations.join")
        .header("Authorization", format!("Bearer {}", bot_token))
        .query(&[("channel", channel_id)])
        .send().expect("Failed to send request");
}