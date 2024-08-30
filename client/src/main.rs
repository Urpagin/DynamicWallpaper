use clap::Parser;
use core::panic;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::{
    collections::{self, HashMap},
    env,
};

use log::{self, debug, error, info, warn};
use reqwest;

#[derive(Parser, Debug)]
#[command(author = "Urpagin")]
struct Args {
    /// DynamicWallpaper endpoint. e.g. https://wallpaper.yourdomain.ext
    #[arg(short, long)]
    endpoint: String,
}

#[tokio::main]
async fn main() {
    init_logging();
    debug!("Program started!");

    let endpoint = read_endpoint();
    let image_endpoint = format!("{endpoint}/images");

    match fetch_image_links(&image_endpoint).await {
        Ok(images) => {
            for img in images {
                info!("{:?}", img);
            }
        }
        Err(e) => {
            error!("Aborting... Failed to fetch images: {e}");
            std::process::exit(-1);
        }
    }
}

/// Reads from args the endpoint and returns it.
fn read_endpoint() -> String {
    let mut result = Args::parse().endpoint.trim().to_string();
    if result.ends_with('/') {
        result.pop();
    }

    if !result.starts_with("http://") || !result.starts_with("https://") {
        error!("Try adding 'http://' or 'https://' before '{result}'");
    }

    result
}

/// Initializes the logging
fn init_logging() {
    let mut builder = env_logger::Builder::new();

    builder.filter_level(log::LevelFilter::Debug);

    builder.init();
}

/// Returns a vec of `Image`s that it fetches from the image endpoint.
async fn fetch_image_links(image_endpoint: &str) -> Result<Vec<Image>, Box<dyn std::error::Error>> {
    debug!("Fetching images with endpoint: '{image_endpoint}'");
    let json: Value = reqwest::get(image_endpoint).await?.json().await?;
    let images_filenames: &Vec<Value> = json["images"].as_array().ok_or("JSON is not an array")?; // A list of filenames (file1.png, file2.jpg)

    let mut result: Vec<Image> = Vec::new();

    for img_filename in images_filenames {
        let filename = img_filename
            .as_str()
            .ok_or("Image filename is not a string")?
            .to_string();

        let download_link: String = format!("{}/{}", image_endpoint, filename);
        result.push(Image {
            download_link,
            filename,
        });
    }

    Ok(result)
}

/// Represents an image with a direct download link and a filename.
#[derive(Debug)]
struct Image {
    download_link: String,
    filename: String,
}

/// Synchronizes the remote images with the local directory.
fn sync_local(directory: &Path, images: Vec<Image>) -> std::io::Result<()> {
    if !directory.is_dir() {
        fs::create_dir(directory)?;
    }

    let mut local: HashSet<String> = HashSet::new();

    let a: HashSet<String> = fs::read_dir(directory)?
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .ok_or("Failed to convert filename to string")
                .unwrap()
        })
        .collect();

    Ok(())
}
