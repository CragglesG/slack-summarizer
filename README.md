# Slack Summarizer

Slack Summarizer is a handy CLI tool for summarizing recent messages in Slack. It allows you to easily catch up on what's happening without leaving your command line.

## Setup

To set up Slack Summarizer, start by creating a Slack app with the `channels:history`, `channels:join`, and `channels:read` scopes. Install the app to your workspace and make a note of the bot token. You'll need it when you first use Slack Summarizer. Now you'll need to get an OpenAI token from your preferred provider. If you choose to use a provider other than OpenAI itself, you'll have to change the default request URL and model name when using Slack Summarizer for the first time. You must ensure that the provider's API is OpenAI-compatible, or Slack Summarizer will be unable to use it.

## First-time Use

When using this tool for the first time, you'll need to set a Slack bot token and OpenAI token. You can do this as below, replacing `SLACK_TOKEN` and `OPENAI_TOKEN` with the appropriate values:

```bash
slack-summarizer -s SLACK_TOKEN -o OPENAI_TOKEN summarize general
```

You can also specify a different OpenAI API URL if you are using a different provider (e.g. Azure OpenAI, GitHub Models):

```bash
slack-summarizer -r API_URL summarize general
```

## Usage


```bash
slack-summarizer [ARGS] summarize CHANNEL
```

Any arguments passed will be written to the configuration file. Existing values will be overwritten. Below is an explanation of all possible flags:

| Long Flag                                           | Short Flag | Description |
| --------------------------------------------------- | ---------- | ----------- |
| `--slack-token` | `-s`       | Set the Slack bot token        |
| `--openai-token` | `-o`       | Set the OpenAI token        |
| `--request-url` | `-r`       | Set the OpenAI request URL        |
| `--model` | `-m`       | Set the OpenAI model       |
| `--tokens` | `-t`       | Set the maximum number of output tokens       |
| `--num-messages` | `-n`       | Set the number of messages to summarize        |