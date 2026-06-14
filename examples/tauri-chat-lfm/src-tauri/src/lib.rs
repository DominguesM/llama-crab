use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle, Manager};

const MODEL_REPO: &str = "LiquidAI/LFM2.5-350M-GGUF";
const MODEL_FILE: &str = "LFM2.5-350M-Q4_K_M.gguf";
const MODEL_URL: &str =
    "https://huggingface.co/LiquidAI/LFM2.5-350M-GGUF/resolve/main/LFM2.5-350M-Q4_K_M.gguf";

#[derive(Clone, Serialize)]
struct DownloadProgress {
    state: &'static str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    path: Option<String>,
}

#[tauri::command]
async fn ensure_lfm_model(
    app: AppHandle,
    on_progress: Channel<DownloadProgress>,
) -> Result<String, String> {
    let model_path = app
        .path()
        .app_local_data_dir()
        .map_err(|error| error.to_string())?
        .join(MODEL_REPO)
        .join(MODEL_FILE);

    if model_path.exists() {
        let metadata = fs::metadata(&model_path).map_err(|error| error.to_string())?;
        send_progress(
            &on_progress,
            "cached",
            metadata.len(),
            Some(metadata.len()),
            Some(&model_path),
        )?;
        return Ok(model_path.to_string_lossy().into_owned());
    }

    tauri::async_runtime::spawn_blocking(move || download_model(&model_path, on_progress))
        .await
        .map_err(|error| error.to_string())?
        .map(|path| path.to_string_lossy().into_owned())
}

fn download_model(
    model_path: &Path,
    on_progress: Channel<DownloadProgress>,
) -> Result<PathBuf, String> {
    let parent = model_path
        .parent()
        .ok_or_else(|| "invalid model path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let tmp_path = model_path.with_extension("gguf.part");
    if tmp_path.exists() {
        fs::remove_file(&tmp_path).map_err(|error| error.to_string())?;
    }

    let mut response = reqwest::blocking::get(MODEL_URL).map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "model download failed with HTTP {}",
            response.status()
        ));
    }

    let total_bytes = response.content_length();
    send_progress(&on_progress, "started", 0, total_bytes, None)?;

    let mut file = File::create(&tmp_path).map_err(|error| error.to_string())?;
    let mut downloaded_bytes = 0;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = response
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|error| error.to_string())?;
        downloaded_bytes += bytes_read as u64;
        send_progress(
            &on_progress,
            "downloading",
            downloaded_bytes,
            total_bytes,
            None,
        )?;
    }

    file.flush().map_err(|error| error.to_string())?;
    fs::rename(&tmp_path, model_path).map_err(|error| error.to_string())?;
    send_progress(
        &on_progress,
        "finished",
        downloaded_bytes,
        total_bytes.or(Some(downloaded_bytes)),
        Some(model_path),
    )?;

    Ok(model_path.to_path_buf())
}

fn send_progress(
    on_progress: &Channel<DownloadProgress>,
    state: &'static str,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    path: Option<&Path>,
) -> Result<(), String> {
    on_progress
        .send(DownloadProgress {
            state,
            downloaded_bytes,
            total_bytes,
            path: path.map(|value| value.to_string_lossy().into_owned()),
        })
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_llama_crab::init())
        .invoke_handler(tauri::generate_handler![ensure_lfm_model])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
