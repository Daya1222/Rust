use clap::{Error, Parser};
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio;
use ytmapi_rs::{YtMusic, query::playlist::GetWatchPlaylistQueryID};

#[derive(Parser, Debug)]
#[command(
    name = "music-player",
    version = "1.0",
    about = "Plays music from YouTube"
)]
struct Args {
    #[arg(short, long)]
    song: String,
}

struct MpvController {
    process: Child,
    pipe_path: String,
}

impl MpvController {
    fn new(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let pipe_path = r"\\.\pipe\mpvsocket";

        // Start mpv with named pipe IPC
        let mut process = Command::new("mpv")
            .args(&[
                "--no-video",
                "--force-window=no",
                &format!("--input-ipc-server={}", pipe_path),
                url,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Give mpv time to create the pipe
        thread::sleep(Duration::from_millis(1000));

        // Check if process is still running
        match process.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("mpv exited early with status: {}", status).into());
            }
            Ok(None) => {
                // Still running, good
            }
            Err(e) => {
                return Err(format!("Error checking mpv status: {}", e).into());
            }
        }

        Ok(MpvController {
            process,
            pipe_path: pipe_path.to_string(),
        })
    }

    fn send_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Try to open the pipe and send command
        match OpenOptions::new().write(true).open(&self.pipe_path) {
            Ok(mut pipe) => {
                let json_command = format!("{}\n", command);
                pipe.write_all(json_command.as_bytes())?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to open pipe: {}", e).into()),
        }
    }

    fn toggle_pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(r#"{"command": ["cycle", "pause"]}"#)
    }

    fn seek(&mut self, seconds: i32) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(&format!(
            r#"{{"command": ["seek", {}, "relative"]}}"#,
            seconds
        ))
    }

    fn set_volume(&mut self, volume: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(&format!(
            r#"{{"command": ["set_property", "volume", {}]}}"#,
            volume
        ))
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_command(r#"{"command": ["quit"]}"#)?;
        thread::sleep(Duration::from_millis(100));
        let _ = self.process.kill();
        Ok(())
    }

    fn is_running(&mut self) -> bool {
        matches!(self.process.try_wait(), Ok(None))
    }
}

impl Drop for MpvController {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

#[tokio::main]
async fn main() {
    let song_name = get_args().expect("Error while getting name");
    println!("Searching for: {}", song_name);

    let (video_id, title) = get_song_url(&song_name)
        .await
        .expect("Error while getting url");
    println!("Found: {} (ID: {})", title, video_id);

    let youtube_url = format!("https://www.youtube.com/watch?v={}", video_id);

    let stream_url = get_audio_stream(&youtube_url);

    if stream_url.is_empty() {
        eprintln!("Error: Failed to get stream URL");
        return;
    }

    println!("Starting playback...");
    let mut controller = match MpvController::new(&stream_url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to start mpv: {}", e);
            return;
        }
    };

    println!("\nControls:");
    println!("  p - pause/play");
    println!("  f - forward 10s");
    println!("  b - backward 10s");
    println!("  ] - volume up");
    println!("  [ - volume down");
    println!("  q - quit");
    println!();

    // Control loop
    use std::io::{BufRead, stdin};
    let stdin = stdin();
    let mut volume = 100;

    for line in stdin.lock().lines() {
        if let Ok(input) = line {
            if !controller.is_running() {
                println!("Playback finished!");
                break;
            }

            match input.trim() {
                "p" => {
                    if let Err(e) = controller.toggle_pause() {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Toggled pause");
                    }
                }
                "f" => {
                    if let Err(e) = controller.seek(10) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Seeked forward 10s");
                    }
                }
                "b" => {
                    if let Err(e) = controller.seek(-10) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Seeked backward 10s");
                    }
                }
                "]" => {
                    volume = (volume + 10).min(150);
                    if let Err(e) = controller.set_volume(volume) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Volume: {}%", volume);
                    }
                }
                "[" => {
                    volume = volume.saturating_sub(10);
                    if let Err(e) = controller.set_volume(volume) {
                        eprintln!("Error: {}", e);
                    } else {
                        println!("Volume: {}%", volume);
                    }
                }
                "q" => {
                    println!("Stopping...");
                    let _ = controller.stop();
                    break;
                }
                "" => {
                    // Empty line, ignore
                }
                _ => println!("Unknown command: {}", input),
            }
        }
    }
}

fn get_args() -> Result<String, Error> {
    let args = Args::parse();
    Ok(args.song)
}

async fn get_song_url(song: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let yt = YtMusic::new_unauthenticated().await?;
    let results = yt.search_songs(song).await?;

    let first_result = results.first().ok_or("No search results found")?;

    let video_id = first_result
        .video_id
        .get_video_id()
        .ok_or("No video ID found")?
        .to_string();

    let title = first_result.title.clone();

    println!("Search results:");
    for (i, result) in results.iter().take(3).enumerate() {
        println!("  {}. {} - {:?}", i + 1, result.title, result.artist);
    }

    Ok((video_id, title))
}

fn get_audio_stream(url: &str) -> String {
    let output = Command::new("yt-dlp")
        .args(&[
            "--ignore-config",
            "--no-cache-dir",
            "-f",
            "bestaudio",
            "-g",
            url,
        ])
        .output()
        .expect("Failed to execute yt-dlp");

    if !output.status.success() {
        eprintln!("yt-dlp failed with status: {}", output.status);
        eprintln!("yt-dlp stderr: {}", String::from_utf8_lossy(&output.stderr));
        return String::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}
