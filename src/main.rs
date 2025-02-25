use crate::types::*;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serenity::all::{CommandDataOptionValue, CreateCommand, CreateInteractionResponseMessage, GatewayIntents, Interaction};
use serenity::async_trait;
use serenity::builder::{CreateCommandOption, CreateInteractionResponse};
use serenity::client::{Client as DiscordClient, Context, EventHandler};
use serenity::model::application::CommandOptionType;
use serenity::model::gateway::Ready;
use std::env;
use std::net::SocketAddr;
use tracing::{error, info};
use warp::Filter;

mod types;

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

                let response_result = self.forward_query(query).await;
                
                match response_result {
                    Ok(resp) => {



                        let message_builder = build_discord_message_from_chat_response ( resp ); 

                       
                        // Send the response with all the components
                        if let Err(e) = command
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(message_builder),
                            )
                            .await
                        {
                            error!("Error responding to slash command: {:?}", e);
                        }


                    },
                    Err(e) => {
                        error!("Error forwarding query: {:?}", e);
                        // Handle error response
                        if let Err(e) = command
                            .create_response(
                                &ctx.http,
                                CreateInteractionResponse::Message(
                                    serenity::builder::CreateInteractionResponseMessage::new()
                                        .content("Sorry, there was an error processing your request.")
                                ),
                            )
                            .await
                        {
                            error!("Error responding to slash command: {:?}", e);
                        }
                    }
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
    async fn forward_query(&self, body: String) -> Result<ChatMessageOutput> {
        // Forward the query to the target service
        let response = self
            .http_client
            .post(&self.target_url)
            .json(&ChatMessageInput { body, api_key: None })
            .send()
            .await?;

        if response.status().is_success() {
            let proxy_response: ChatMessageOutput = response.json().await?;
            Ok(proxy_response)
        } else {
            let status = response.status();
            let text = response.text().await?;
            error!("Error from target service: {} - {}", status, text);
            Err(anyhow!("Error from target service: {} - {}", status, text))
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
    let target_url = env::var("PROXY_TARGET_URL").expect("Expected a target URL in the environment");

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



// ------------- 

fn build_discord_message_from_chat_response( resp: ChatMessageOutput )
             -> CreateInteractionResponseMessage {



                  let mut message_builder = serenity::builder::CreateInteractionResponseMessage::new()
                            .content(resp.body);
                        
                        // Check if we have transaction data to display
                        if let Some(tx_array) = resp.tx_array {
                            if !tx_array.is_empty() {
                                // Create a table or formatted block for transactions
                                let mut tx_formatted = String::from("```\nTransaction Details:\n");
                                tx_formatted.push_str("----------------------------------------\n");
                                
                                for (i, tx) in tx_array.iter().enumerate() {
                                    tx_formatted.push_str(&format!("Transaction #{}\n", i + 1));
                                    tx_formatted.push_str(&format!("Chain ID: {}\n", tx.chain_id));
                                    tx_formatted.push_str(&format!("To: {}\n", tx.to_address));
                                    
                                    if let Some(desc) = &tx.description {
                                        tx_formatted.push_str(&format!("Description: {}\n", desc));
                                    }
                                    
                                    tx_formatted.push_str("----------------------------------------\n");
                                }
                                
                                tx_formatted.push_str("```");
                                message_builder = message_builder.add_embed(
                                    serenity::builder::CreateEmbed::new()
                                        .title("Transactions")
                                        .description(tx_formatted)
                                        .color(0x00FF00)
                                );
                            }
                        }
                        
                        // Check if we have structured data to display
                        if let Some(structured_data) = resp.structured_data {
                            // Convert structured data to a formatted string
                            let formatted_data = match serde_json::to_string_pretty(&structured_data) {
                                Ok(json_str) => format!("```json\n{}\n```", json_str),
                                Err(_) => String::from("```\nUnable to format structured data\n```")
                            };
                            
                            message_builder = message_builder.add_embed(
                                serenity::builder::CreateEmbed::new()
                                    .title("Additional Data")
                                    .description(formatted_data)
                                    .color(0x0000FF)
                            );
                        }


                    message_builder
                        

}