# Discord Proxy Bot (Rust)

A simple Discord bot built with Serenity framework that acts as a proxy, forwarding user queries to another service and returning the responses.

## Features

- Slash command `/bot` that forwards user queries to a specified endpoint
- Dockerized for easy deployment
- Ready for DigitalOcean App Platform

## Setup

### Prerequisites

- Rust (latest stable)
- Discord Bot Token
- Endpoint URL for forwarding requests

### Environment Variables

Create a `.env` file in the root directory with the following variables:

```
DISCORD_TOKEN=your_discord_bot_token
TARGET_URL=https://your-target-service.com/api/endpoint
```

### Discord Bot Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Navigate to the "Bot" tab and create a bot
4. Enable the following Privileged Gateway Intents:
   - Message Content Intent
5. Copy the bot token and add it to your `.env` file
6. Use the OAuth2 URL Generator with the following scopes:
   - `bot`
   - `applications.commands`
7. Invite the bot to your server

## Local Development

1. Clone this repository
2. Set up your `.env` file with the required variables
3. Run the bot:

```bash
cargo run
```

## Building the Docker Image

```bash
docker build -t discord-proxy-bot .
```

## Running with Docker

```bash
docker run -e DISCORD_TOKEN=your_token -e TARGET_URL=your_url discord-proxy-bot
```

## Deploying to DigitalOcean App Platform

1. Fork or push this repository to GitHub
2. In the DigitalOcean dashboard, create a new App
3. Select your repository
4. Configure the environment variables:
   - `DISCORD_TOKEN`: Your Discord bot token
   - `TARGET_URL`: The URL to forward requests to
5. Deploy the app

### Using the App Spec

You can also use the included `app.yaml` file to deploy directly with the DigitalOcean CLI:

```bash
doctl apps create --spec app.yaml
```

## Usage

Once the bot is running and added to your server, you can use the slash command:

```
/bot your query here
```

The bot will forward "your query here" to the target service and respond with the result.

## Expected API Response Format

The target service should return a JSON response with the following structure:

```json
{
  "response": "This is the response text that will be sent back to Discord."
}
```

## Adding Health Check (Optional)

For a more robust deployment, you can add a simple health check endpoint:

1. Add the `warp` package to your dependencies
2. Implement a simple HTTP server with a `/health` endpoint
3. Make sure your app listens on the port specified in the App Platform config (default: 8080)

## License

MIT
