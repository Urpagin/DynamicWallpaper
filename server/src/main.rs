use axum::http::HeaderMap;
use convert_case::{Case, Casing};
use core::panic;
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

use axum::http::header;

use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use tokio::{fs::File, io::AsyncWriteExt};

use thiserror::Error;

use serde_json::{json, Value};
use uuid::Uuid;

/// This hashmap holds hashes for the files, so that twice the same file cannot be saved.
static FILE_HASHES: Lazy<Mutex<HashMap<PathBuf, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const ADDRESS: &str = "0.0.0.0:4000";
const IMAGE_DIRECTORY: &str = "wallpapers";

const MAX_FILE_SIZE_BYTES: u64 = 1024 * 1024 * 30; // 30MiB
const MAX_FILE_NAME_LENGTH: u64 = 255;

#[tokio::main]
async fn main() {
    setup_logging(log::LevelFilter::Debug);
    debug!("App started.");

    init_image_directory(IMAGE_DIRECTORY).unwrap_or_else(|e| {
        error!("Failed to create the directory or compute the file hashes: {e}");
        panic!("Failed to create the directory or compute the file hashes: {e}");
    });

    let app = Router::new()
        .route("/", get(serve_file))
        .route("/upload", post(upload_file))
        .route("/images", get(get_images))
        .route("/images/:filename", get(serve_image))
        .route("/delete/:filename", axum::routing::delete(delete_image))
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE_BYTES as usize));

    let listener = tokio::net::TcpListener::bind(ADDRESS)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to bind to address {ADDRESS}: {e}");
            panic!("Could not start server: {e}");
        });

    info!("LISTENING ON {ADDRESS}");

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("Server error: {e}");
        panic!("Server crashed: {e}");
    });
}

async fn serve_file() -> impl IntoResponse {
    match tokio::fs::read_to_string("assets/index.html").await {
        Ok(content) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "text/html".parse().unwrap());
            (StatusCode::OK, headers, content).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found".to_string()).into_response(),
    }
}

fn setup_logging(level: log::LevelFilter) {
    env_logger::Builder::new().filter(None, level).init();
}

// TODO: Delete same hashes at start, just to be sure?. (or maybe don't)
fn init_image_directory(image_directory: &str) -> Result<(), std::io::Error> {
    // Making the image directory if it already doesn't exist.
    match fs::create_dir(image_directory) {
        Ok(_) => {
            info!("Successfully created directory: {IMAGE_DIRECTORY}");
        }
        Err(e) => {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                error!("Failed to create directory: {IMAGE_DIRECTORY}");
                panic!("{e}");
            }
        }
    }

    if let Err(e) = compute_initial_digests(image_directory) {
        error!("Failed to compute image directory initial digests: {e}");
        return Err(e);
    }

    Ok(())
}

/// Computes the SHA-256 digests of each file in the image directory and populates the global hash
/// HashMap.
fn compute_initial_digests(image_directory: &str) -> Result<(), std::io::Error> {
    info!("Computing initial file digests... (might take some time)");

    // Computing file hashes
    let start = std::time::Instant::now();
    let mut file_count: usize = 0;
    let mut total_megabytes_hashed: f64 = 0.0;

    let entries = fs::read_dir(image_directory)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        file_count += 1;
        let file_hash = compute_file_hash(path.to_str().expect("Error path to_str()"))?;
        total_megabytes_hashed += file_hash.1 as f64 / 1024.0 / 1024.0;

        // Insert filepath: filehash into the global HashSet
        {
            let mut map = FILE_HASHES.lock().unwrap();
            map.insert(path, file_hash.0);
        }
    }

    let elapsed_time = (start.elapsed().as_secs_f64() * 10.0).round() / 10.0;
    let rounded_megabytes = (total_megabytes_hashed * 10.0).round() / 10.0;

    info!(
        "Computed the hash of {} files in {:#?}s, totaling {:#?} MiB",
        file_count, elapsed_time, rounded_megabytes
    );

    Ok(())
}

/// This functions computes the hash of a file using the Sha256 hashing algorithm.
/// This function does not put the file all into memory.
/// This function returns a tuple (File Hash, File Size (in bytes))
fn compute_file_hash(path: &str) -> Result<(String, u64), std::io::Error> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)?;

    let bytes_hashed = std::io::copy(&mut file, &mut hasher)?;
    let hash_bytes = hasher.finalize();
    Ok((hex::encode(hash_bytes), bytes_hashed))
}

/// Adds the digest of a file into the global hash HashMap.
fn add_digest(path: &str) -> Result<(), std::io::Error> {
    match compute_file_hash(path) {
        Ok(digest) => {
            let path = PathBuf::from(path);
            FILE_HASHES.lock().unwrap().insert(path.clone(), digest.0);
            info!("Added hash of file '{:?}'", path);
            Ok(())
        }
        Err(e) => {
            error!("Failed to compute file digest: {e}");
            Err(e)
        }
    }
}

/// Checks the digest of the input file against the global hash HashMap.
fn is_file_duplicate(path: &str) -> Result<bool, std::io::Error> {
    let digest = compute_file_hash(path)?;
    Ok(FILE_HASHES
        .lock()
        .unwrap()
        .values()
        .any(|hash| hash == &digest.0))
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to process multipart form")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("File is not an image")]
    NotAnImage,
    #[error("Filename too long")]
    FilenameTooLong,
    #[error("File too large (max is 30MB)")]
    FileTooLarge,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::MultipartError(_) => (StatusCode::BAD_REQUEST, "Invalid form data"),
            Self::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Server error"),
            Self::NotAnImage => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "File is not an image"),
            Self::FilenameTooLong => (StatusCode::BAD_REQUEST, "Filename too long"),
            Self::FileTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "File too large (max is 30MB)",
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

fn is_valid_image_extension(filename: &str) -> bool {
    if filename.split('.').count() < 2 {
        return false;
    }

    let extension = filename.split('.').last().unwrap_or("").to_lowercase();

    matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp")
}

fn generate_filename(filename_input: &str) -> Result<String, AppError> {
    let filename = sanitize(filename_input);

    // This is beautiful, I love this one-liner!
    let extension = filename
        .split('.')
        .last()
        .ok_or(AppError::NotAnImage)?
        .to_lowercase();

    info!("extension={extension}");

    let file_stem = Path::new(&filename)
        .file_stem()
        .ok_or(AppError::NotAnImage)?
        .to_str()
        .ok_or(AppError::NotAnImage)?;

    let random_uuid = Uuid::new_v4();

    let filename = format!("{}-{}.{}", file_stem, random_uuid, extension);

    if filename.len() as u64 > MAX_FILE_NAME_LENGTH {
        Err(AppError::FilenameTooLong)
    } else {
        Ok(filename)
    }
}

/// This functions "sanitizes" a string. It removes spaces and non-ASCII characters.
/// Ideal for making a good file name.
fn sanitize(text: &str) -> String {
    let mut result = text.to_case(Case::Snake);
    result = result.chars().filter(|&c| c.is_ascii()).collect();
    result
}

async fn upload_file(mut multipart: Multipart) -> Result<impl IntoResponse, AppError> {
    info!("Starting file upload process");
    while let Some(field) = multipart.next_field().await? {
        debug!("Processing new field from multipart form");
        if field.name() != Some("wallpaper") {
            error!("Invalid field name: {:#?}", field.name());
            return Err(AppError::NotAnImage);
        }

        let received_filename = field.file_name().ok_or_else(|| {
            error!("No filename provided");
            AppError::NotAnImage
        })?;

        if received_filename.is_empty() {
            warn!("Filename is empty, aborting upload");
            return Err(AppError::NotAnImage);
        }

        info!("Received file: {}", received_filename);

        if !is_valid_image_extension(received_filename) {
            error!("Invalid file extension: {}", received_filename);
            return Err(AppError::NotAnImage);
        }

        let final_filename = generate_filename(received_filename)?;
        let final_filepath = Path::new(IMAGE_DIRECTORY).join(&final_filename);
        debug!("Final filepath: {:#?}", final_filepath);

        let file = File::create_new(&final_filepath).await.map_err(|e| {
            error!("Failed to create new file: {:?}", final_filepath);
            e
        })?;

        match upload_and_save(field, file).await {
            Ok(_) => {
                info!("Successfully saved file: {:?}", final_filepath);
                Ok(())
            }
            Err(e) => {
                error!("Failed to put data in file: {e}");
                if let Err(remove_err) = fs::remove_file(&final_filepath) {
                    error!("Failed to remove the partial file: {remove_err}");
                }
                Err(e)
            }
        }?;

        // Check for duplicates
        if is_file_duplicate(&final_filepath.to_string_lossy())? {
            info!("Found duplicate: {:?}", &final_filepath);
            tokio::fs::remove_file(&final_filepath).await?;
            debug!("Removed duplicate file: {:?}", final_filepath);
        } else {
            add_digest(&final_filepath.to_string_lossy())?;
            debug!("Added file digest {:?} to HashMap", final_filepath);
        }
    }

    Ok(Json(json!({"message": "File uploaded successfully"})))
}

/// Uploads and then saves the file onto the machine's fs.
async fn upload_and_save(
    mut field: axum::extract::multipart::Field<'_>,
    mut file: File,
) -> Result<(), AppError> {
    let mut file_size: u64 = 0;
    while let Some(chunk) = field.chunk().await? {
        file_size += chunk.len() as u64;
        if file_size > MAX_FILE_SIZE_BYTES {
            debug!("{file_size} & {MAX_FILE_SIZE_BYTES}");
            return Err(AppError::FileTooLarge);
        }
        file.write_all(&chunk).await?;
    }

    file.sync_all().await?;

    Ok(())
}

/// Returns a JSON of all of the accessible image paths. (json of such: website.com/images/img_1.png)
async fn get_images() -> Result<Json<Value>, StatusCode> {
    let mut images = Vec::new();
    let mut entries = tokio::fs::read_dir(IMAGE_DIRECTORY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if let Ok(file_type) = entry.file_type().await {
            if file_type.is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    images.push(file_name.to_string());
                }
            }
        }
    }

    Ok(Json(json!({"images": images})))
}

/// Provides a shortcut from addr/wallpapers/img.jpg to addr/images/img.jpg
async fn serve_image(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> impl IntoResponse {
    let path = PathBuf::from(IMAGE_DIRECTORY).join(&filename);
    match tokio::fs::read(path).await {
        Ok(contents) => {
            let content_type = mime_guess::from_path(&filename).first_or_octet_stream();
            ([(header::CONTENT_TYPE, content_type.as_ref())], contents).into_response()
        }
        Err(_) => Json(json!({"error": "Image not found"})).into_response(),
    }
}

/// Deletes the image on the fs from its path.
async fn delete_image(
    axum::extract::Path(filename): axum::extract::Path<String>,
) -> impl IntoResponse {
    let file_path = Path::new(IMAGE_DIRECTORY).join(&filename);
    match fs::remove_file(file_path) {
        Ok(_) => {
            let key = PathBuf::from(format!(
                "{IMAGE_DIRECTORY}/{}",
                filename.split('/').last().unwrap()
            ));

            // Remove the hash from FILE_HASHES
            if FILE_HASHES.lock().unwrap().remove(&key).is_some() {
                debug!("Removed hash with file: {:?}", filename);
            } else {
                debug!("Failed to remove hash with file: {:?}", filename);
            }

            info!("Removed file: {:?}", filename);
            Json(json!({"message": "Image deleted successfully"})).into_response()
        }
        Err(_) => Json(json!({"error": "Image not found"})).into_response(),
    }
}
