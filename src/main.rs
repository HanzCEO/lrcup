use clap::{Parser, Subcommand};
use reqwest::Client;
use serde_json;

mod challenge;

#[derive(Parser)]
#[command(name = "lrcup")]
#[command(about = "LRCLIB Lyrics Uploader", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a file
    Upload {
        /// Path to the file to upload
        filepath: String,
    },
}

use std::io::{self, Write};

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Failed to read input");
    buffer.trim().to_string()
}

async fn publish(filepath: &str, token: &str) {
    let track_name = input("Enter track name: ");
    let artist_name = input("Enter artist name: ");
    let album_name = input("Enter album name: ");
    let duration = match input("Enter duration (in seconds): ").parse::<u32>() {
        Ok(secs) => secs,
        Err(_) => {
            let time = input("Invalid duration input. Enter duration as MM:SS: ");
            let parts: Vec<&str> = time.split(':').collect();
            if parts.len() == 2 {
                let minutes = parts[0].parse::<u32>().unwrap_or(0);
                let seconds = parts[1].parse::<u32>().unwrap_or(0);
                minutes * 60 + seconds
            } else {
                panic!("Invalid MM:SS format");
            }
        }
    };
    let plain_lyrics = "";
    let synced_lyrics = std::fs::read_to_string(filepath).expect("Failed to read file");

    let client = Client::new();
    let resp = client
        .post("https://lrclib.net/api/publish")
        .body(serde_json::json!({
            "trackName": track_name,
            "artistName": artist_name,
            "albumName": album_name,
            "duration": duration,
            "plainLyrics": plain_lyrics,
            "syncedLyrics": synced_lyrics
        }).to_string())
        .header("X-Publish-Token", token)
        .send()
        .await;
    if resp.is_ok() {
        println!("File uploaded successfully!");
    } else {
        eprintln!("Failed to upload file: {:?}", resp.err());
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Upload { filepath } => {
            println!("Uploading file: {}", filepath);
            // TODO: implement upload logic
            let chal = challenge::request_challenge().await;
            match chal {
                Ok(response) => {
                    println!("Challenge received: {:?}", response);
                    let nonce = challenge::solve_challenge(&response.prefix, &response.target);
                    println!("Nonce found: {}", nonce);
                    let token = format!("{}:{}", response.prefix, nonce);
                    publish(&filepath, &token).await;
                }
                Err(e) => {
                    eprintln!("Error requesting challenge: {}", e);
                }
            }
        }
    }
}
