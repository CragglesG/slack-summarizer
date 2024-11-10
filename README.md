# Slack Summarizer

Slack Summarizer is a handy CLI tool for summarizing recent messages in Slack. It allows you to easily catch up on what's happening without leaving your command line.

## Setup

To set up Slack Summarizer, start by creating a Slack app with the `channels:history`, `channels:join`, and `channels:read` scopes. Install the app to your workspace and make a note of the bot token. You'll need it when you first use Slack Summarizer. Now you'll need to get an OpenAI token from your preferred provider. If you choose to use a provider other than OpenAI itself, you'll have to change the default request URL when using Slack Summarizer for the first time.

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

Optional values are indicated in brackets:

```bash
slack-summarizer [--slack-bot-token/-s TOKEN] [--openai-token/-o TOKEN] [--request-url/-r URL] summarize CHANNEL
```