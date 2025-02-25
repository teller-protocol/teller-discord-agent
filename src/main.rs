use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serenity::all::{CommandDataOptionValue, CreateCommand, GatewayIntents, Interaction};
use serenity::async_trait;
use serenity::builder::{CreateCommandOption, CreateInteractionResponse};
use serenity::client::{Client as DiscordClient, Context, EventHandler};
use serenity::model::application::CommandOptionType;
use serenity::model::gateway::Ready;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use warp::Filter;

// Structure for the POST request
#[derive(Serialize)]
struct ProxyRequest {
    query: String,
}

// Structure for the POST response
#[derive(Deserialize, Debug)]
struct ProxyResponse {
    response: String,
}

struct Handler {
    http_client: Client,
    target_url: String,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received command: {}", command.data.name);

            if command.data.name.as_str() == "bot" {
                let mut query = String::new();

                // Extract the query from the command options
                if let Some(option) = command.data.options.first() {
                    if let CommandDataOptionValue::String(s) = &option.value {
                        query = s.clone();
                    }
                }

                info!("Forwarding query: {}", query);

                let response = match self.forward_query(query).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        error!("Error forwarding query: {:?}", e);
                        "Sorry, there was an error processing your request.".to_string()
                    }
                };

                // Respond to the Discord interaction
                if let Err(e) = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            serenity::builder::CreateInteractionResponseMessage::new()
                                .content(response),
                        ),
                    )
                    .await
                {
                    error!("Error responding to slash command: {:?}", e);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Create the slash command
        if let Err(e) = ctx.http.create_global_command(
            &CreateCommand::new("bot")
                .description("Forward a query to the target service")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "query",
                        "The query to forward",
                    )
                    .required(true),
                ),
        )
        .await
        {
            error!("Error creating command: {:?}", e);
        }
    }
}

impl Handler {
    async fn forward_query(&self, query: String) -> Result<String> {
        // Forward the query to the target service
        let response = self
            .http_client
            .post(&self.target_url)
            .json(&ProxyRequest { query })
            .send()
            .await?;

        if response.status().is_success() {
            let proxy_response: ProxyResponse = response.json().await?;
            Ok(proxy_response.response)
        } else {
            let status = response.status();
            let text = response.text().await?;
            error!("Error from target service: {} - {}", status, text);
            Ok(format!(
                "The target service returned an error: {} - {}",
                status, text
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Initialize the logger
    tracing_subscriber::fmt::init();

    // Discord bot token
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    
    // Target URL for forwarding requests
    let target_url = env::var("TARGET_URL").expect("Expected a target URL in the environment");

    let http_client = Client::new();

    // Create a health check for DigitalOcean App Platform
    let health_route = warp::path("health").map(|| "OK");
    
    // Get the port from the environment or use default
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("PORT must be a number");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    // Serve the health check endpoint in a separate task
    let health_server = tokio::spawn(async move {
        info!("Starting health check server on port {}", port);
        warp::serve(health_route).run(addr).await;
    });

    // Create a new serenity client
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = DiscordClient::builder(&token, intents)
        .event_handler(Handler {
            http_client,
            target_url,
        })
        .await?;

    // Start the client
    info!("Starting Discord bot...");
    
    // Run both the bot and the health check server
    tokio::select! {
        result = client.start() => {
            if let Err(why) = result {
                error!("Client error: {:?}", why);
            }
        }
        _ = health_server => {
            error!("Health check server stopped unexpectedly");
        }
    }
    
    Ok(())
}
