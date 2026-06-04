#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_shell::ShellExt;
use tauri::Emitter;
use regex::Regex;

#[derive(Clone, serde::Serialize)]
struct DownloadProgress {
    percent: String,
    speed: String,
    eta: String,
    status: String,
}

#[tauri::command]
async fn download_video(app_handle: tauri::AppHandle, url: String, download_path: String) -> Result<String, String> {
    let output_template = format!("{}/%(title)s.%(ext)s", download_path);
    
    // Logika penentuan format berdasarkan domain URL
    // Buka src-tauri/src/main.rs dan cari bagian ini:
    let format_arg = if url.contains("youtube.com") || url.contains("youtu.be") {
        // Lock ke 720p mp4 langsung jadi yang ramah QuickTime
        "best[ext=mp4][vcodec^=avc1]/best[ext=mp4]/best"
    } else {
        // Untuk Instagram, TikTok, X: Paksa cari yang video-nya h264 (avc1) dan audio-nya aac
        // Jika tidak ada, dia akan mundur ke mp4 terbaik yang tersedia.
        "bestvideo[ext=mp4][vcodec^=avc1]+bestaudio[ext=m4a][acodec^=mp4a]/best[ext=mp4][vcodec^=avc1]/best[ext=mp4]/best"
    };

    let (mut rx, _child) = app_handle
        .shell()
        .sidecar("yt-dlp")
        .map_err(|e| e.to_string())?
        .args(&[
            &url, 
            "-o", &output_template, 
            "-f", format_arg,
            "--no-playlist"
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let progress_regex = Regex::new(r"\[download\]\s+([0-9.]+)%\s+of\s+.+?\s+at\s+(.+?)\s+ETA\s+(.+)").unwrap();

    while let Some(event) = rx.recv().await {
        if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
            let output_str = String::from_utf8_lossy(&line);
            let trimmed = output_str.trim();
            println!("{}", trimmed); 

            if let Some(caps) = progress_regex.captures(trimmed) {
                let percent = caps.get(1).map_or("0", |m| m.as_str()).to_string();
                let speed = caps.get(2).map_or("-", |m| m.as_str()).to_string();
                let eta = caps.get(3).map_or("--:--", |m| m.as_str()).to_string();

                let _ = app_handle.emit("download-progress", DownloadProgress {
                    percent,
                    speed,
                    eta,
                    status: "Mengunduh file...".to_string(),
                });
            } else if trimmed.contains("[download] Destination:") {
                let _ = app_handle.emit("download-progress", DownloadProgress {
                    percent: "0".to_string(),
                    speed: "-".to_string(),
                    eta: "-".to_string(),
                    status: "Menghubungkan ke server media...".to_string(),
                });
            }
        }
    }

    Ok("Download Selesai dan Berhasil Disimpan!".to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![download_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}