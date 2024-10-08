use clap::Parser;
use reqwest::Client;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::{collections::HashSet, io::Cursor};

use log::{self, debug, error, info, warn};

#[derive(Parser, Debug)]
#[command(author = "Urpagin")]
struct Args {
    /// DynamicWallpaper endpoint. e.g. https://wallpaper.yourdomain.ext
    #[arg(short, long)]
    endpoint: String,

    /// Directory where the images will be synced. (do not choose a directory where there
    /// already are files, THEY WILL BE REMOVED)
    #[arg(short, long)]
    directory: String,

    /// (optional) If you used the default NGINX proxy config specify user and password.
    #[arg(short, long)]
    user: Option<String>,

    /// (optional) If you used the default NGINX proxy config specify user and password.
    #[arg(short, long)]
    password: Option<String>,
}

#[tokio::main]
async fn main() {
    init_logging();
    debug!("Program started!");

    let args = Args::parse();
    let endpoint: String = read_endpoint(&args);
    let image_directory: String = read_image_directory(&args);
    let auth: Option<Authentication> = read_auth(&args);

    let image_endpoint: String = format!("{endpoint}/images");

    match fetch_image_links(&image_endpoint, &auth).await {
        Ok(images) => {
            for img in &images {
                info!("{:?}", img);
            }
            if let Err(e) = sync_local(image_directory, images, &auth).await {
                error!("Failed to sync local with remote: {e}");
            }
        }
        Err(e) => {
            error!("Failed to fetch images: {e}");
            warn!(
                "You might consider using arguments --user and --password for NGINX's basic auth"
            );
        }
    }
}

/// Reads from args the endpoint and returns it.
fn read_endpoint(args: &Args) -> String {
    let mut result = args.endpoint.trim().to_string();
    if result.ends_with('/') {
        result.pop();
    }

    if !result.starts_with("http://") && !result.starts_with("https://") {
        error!("Try adding 'http://' or 'https://' before '{result}'");
    }

    result
}

/// Reads from args the image directory and returns it.
fn read_image_directory(args: &Args) -> String {
    let mut result = args.directory.trim().to_string();
    if result.ends_with('/') {
        result.pop();
    }
    result
}

/// Reads from args to get user, password for auth.
fn read_auth(args: &Args) -> Option<Authentication> {
    if let (Some(user), Some(password)) = (args.user.as_ref(), args.password.as_ref()) {
        Some(Authentication {
            user: user.to_string(),
            password: password.to_string(),
        })
    } else {
        None
    }
}

/// Initializes the logging
fn init_logging() {
    let mut builder = env_logger::Builder::new();

    builder.filter_level(log::LevelFilter::Debug);

    builder.init();
}

/// If the server side is password-protected by the provided NGINX proxy config file,
/// any request to the server asks for a user and a password.
#[derive(Clone)]
struct Authentication {
    user: String,
    password: String,
}

/// Returns a vec of `Image`s that it fetches from the image endpoint.
async fn fetch_image_links(
    image_endpoint: &str,
    auth: &Option<Authentication>,
) -> Result<Vec<Image>, Box<dyn std::error::Error>> {
    debug!("Fetching images with endpoint: '{image_endpoint}'");

    // Handle auth (NGINX proxy) or no auth (no NGINX proxy).
    let response: reqwest::Response;
    if let Some(auth) = auth {
        response = Client::new()
            .get(image_endpoint)
            .basic_auth(auth.user.clone(), Some(auth.password.clone()))
            .send()
            .await?;
    } else {
        response = reqwest::get(image_endpoint).await?;
    }

    let json: Value = response.json().await?;
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
async fn sync_local<P>(
    directory: P,
    images: Vec<Image>,
    auth: &Option<Authentication>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    ensure_directory_exists(&directory)?;

    let local_filenames: HashSet<String> = fs::read_dir(&directory)?
        .map(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_str()
                .ok_or("Failed to convert filename to string")
                .unwrap()
                .to_string()
        })
        .collect();

    // Add images
    let mut tasks: Vec<_> = Vec::new();

    let mut images_filenames: HashSet<String> = HashSet::new();
    for img in images {
        images_filenames.insert(img.filename.clone());
        if !local_filenames.contains(&img.filename) {
            let path = Path::new(directory.as_ref()).join(&img.filename);
            let auth_clone = auth.clone();
            tasks.push(tokio::spawn(async move {
                match download_file(&path, &img.download_link, &auth_clone).await {
                    Ok(_) => {
                        info!("Successfully downloaded: {:?}", path);
                    }
                    Err(e) => {
                        error!("Failed to download: {:?} at {}: {}", path, &img.filename, e);
                    }
                }
            }))
            //download_file(path, &img.download_link, auth).await?;
        }
    }

    // Download images asynchronously (nice)
    futures::future::join_all(tasks).await;

    // Remove images
    for local_filename in local_filenames {
        if !images_filenames.contains(&local_filename) {
            let path = Path::new(directory.as_ref()).join(local_filename);
            remove_file(path)?;
        }
    }

    Ok(())
}

fn ensure_directory_exists<P: AsRef<Path>>(directory: P) -> std::io::Result<()> {
    let path = directory.as_ref();
    if !path.is_dir() {
        fs::create_dir_all(path)?;
        info!("Created directory: {:?}", path);
    }
    Ok(())
}

/// Downloads a file from a URL and saves it at `path` which also contains the file name
async fn download_file<P>(
    path: P,
    url: &str,
    auth: &Option<Authentication>,
) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let client = Client::new();
    let mut request = client.get(url);

    if let Some(auth) = auth {
        request = request.basic_auth(&auth.user, Some(&auth.password));
    }

    let response = request.send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", response.status()).into());
    }

    let mut file = std::fs::File::create(&path)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    info!("Downloaded file: {:?}", path);
    Ok(())
}

/// Removes a file from the fs using its path.
fn remove_file<P: AsRef<Path> + std::fmt::Debug>(path: P) -> std::io::Result<()> {
    fs::remove_file(&path)?;
    info!("Removed file: {:?}", path);
    Ok(())
}
