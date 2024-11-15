use std::{collections::HashMap, fs::File, path::Path};

use clap::{Parser, Subcommand};

use reqwest::blocking::Client;

use serde_json::{Value, json, to_writer};

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

    /// Refill the channel list
    #[arg(short, long)]
    channels_refill: bool,

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

    let mut channels: HashMap<String, String> = HashMap::new();
    if args.channels_refill {
        channels = get_channels(slack_token.clone(), true);
    } else {
        channels = get_channels(slack_token.clone(), false);
    }
    let messages = if let Some(Commands::Summarize { channel }) = args.command {
        match channels.get(&channel) {
            Some(channel_id) => get_messages(slack_token.clone(), channel_id.clone(), num_messages),
            None => {
                eprintln!("Channel '{}' not found.", channel);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    let summary = summarize_messages(messages, openai_token.clone(), request_url.clone(), model.clone(), max_tokens.clone());
    let cleaned_summary = summary.replace("\\n", "\n").replace("\\", "") + "\n";
    print_inline(cleaned_summary.as_str());

    confy::store("slack-summarizer", None, Config { slack_token:slack_token, openai_token:openai_token, request_url:request_url, model:model, max_tokens:max_tokens, num_messages:num_messages }).expect("Failed to store configuration");

}

fn get_channels(bot_token: String, force_refill: bool) -> HashMap<String, String> {
    let mut channels = HashMap::new();

    let path = confy::get_configuration_file_path("slack-summarizer", None).unwrap_or("".into()).into_os_string().into_string().unwrap_or("".into());

    if !force_refill && Path::new(&(path.clone() + "channels.json")).exists() {
        let hashmap_file = File::open(path + "channels.json").expect("Failed to open file");
        channels = serde_json::from_reader(hashmap_file).expect("Failed to read file");
        return channels;
    }

    let client = Client::new();
    let response = client
        .get("https://slack.com/api/conversations.list")
        .header("Authorization", format!("Bearer {}", bot_token))
        .query(&[("types", "public_channel"), ("limit", "1000"), ("exclude_archived", "false")])
        .send().expect("Failed to send request");

    let mut response_json: Value = response.json().expect("Failed to parse JSON");

    for channel in response_json["channels"].as_array().unwrap() {
        channels.insert(
            channel["name"].as_str().unwrap().to_string(),
            channel["id"].as_str().unwrap().to_string(),
        );
    }

    while response_json["response_metadata"]["next_cursor"].as_str().is_some() {
        let cursor = response_json["response_metadata"]["next_cursor"].as_str().unwrap();
        let response = client
            .get("https://slack.com/api/conversations.list")
            .header("Authorization", format!("Bearer {}", bot_token))
            .query(&[("types", "public_channel"), ("limit", "1000"), ("exclude_archived", "true"), ("cursor", cursor)])
            .send().expect("Failed to send request");

        response_json = response.json().unwrap_or_else(|_| json!({"channels": [], "response_metadata": {}}));

        for channel in response_json["channels"].as_array().unwrap_or(&Vec::new()) {
            channels.insert(
                channel["name"].as_str().unwrap().to_string(),
                channel["id"].as_str().unwrap().to_string(),
            );
        }
    }

    let hashmap_file = File::create("channels.json").expect("Failed to create file");
    to_writer(hashmap_file, &channels).expect("Failed to write to file");

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
        .query(&json!({"channel": channel_id}))
        .send().expect("Failed to send request");
}