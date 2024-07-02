use std::fmt::format;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::{env, fs};

use serenity::all::{CreateAttachment, CreateMessage};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
    
        match fs::remove_file("converted.mp4") {
            Ok(_) => {},
            Err(_) => {},
        }
            
        match fs::remove_file("download.webm") {
            Ok(_) => {},
            Err(_) => {},
        }
            
        let attachment = match msg.attachments.get(0) {

            Some(attachment) => {
                attachment                
            },
            None => {
                println!("no attachments");
                return;
            },
        };

        if !attachment.filename.ends_with(".webm") {
            println!("attachment was not a webm");
            return;
        }

        msg.channel_id.broadcast_typing(&ctx.http).await;

        let attachment_bytes = match attachment.download().await {
            Ok(attachment_bytes) => attachment_bytes,
            Err(error) => {
                msg.channel_id.say(&ctx.http, format!("Failed to download attachment: {}", error.to_string())).await;
                return;
            },
        };

    
        match fs::write("download.webm", attachment_bytes) {
            Ok(_) => {},
            Err(error) => {
                println!("failed to save attachment to disk: {}", error.to_string());
                return;
            },
        }

        match Command::new("ffmpeg")
            .arg("-i")
            .arg("download.webm")
            .arg("converted.mp4")
            .output()
        {
            Ok(_) => {},
            Err(error) => {
                msg.channel_id.say(&ctx.http, format!("Failed to convert webm to mp4: ```{}```", error.to_string())).await;
            },
        }

        let converted_bytes = match fs::read("converted.mp4") {
            Ok(converted_bytes) => converted_bytes,
            Err(error) => {
                println!("failed to read converted file from disk: {}", error.to_string());

                return;
                
            },
        };

        let converted_attachment = CreateAttachment::bytes(converted_bytes, "converted.mp4");
        let reply = CreateMessage::new()
            .add_file(converted_attachment)
            .reference_message(&msg);
        
        match msg.channel_id.send_message(&ctx.http, reply).await {
            Ok(_) => {},
            Err(error) => {
                println!("failed to send conversion: {}", error.to_string());

                return;
            },
        }

        
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("AUTO_CONVERTER_DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}