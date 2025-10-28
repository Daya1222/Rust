use std::process::Command;
use tokio;
use ytmapi_rs::{YtMusic, query::playlist::GetWatchPlaylistQueryID};

#[tokio::main]
async fn main() -> Result<(), ytmapi_rs::Error> {
    let yt = YtMusic::new_unauthenticated().await?;
    let results = yt.search_songs("Yellow").await?;

    let output = if let Some(first) = results.first() {
        match first.video_id.get_video_id() {
            Some(id) => Command::new("yt-dlp")
                .args([
                    "-f",
                    "bestaudio",
                    "-g",
                    &format!("http://www.youtube.com/watch?v={}", id),
                ])
                .output()
                .expect("Failed to run yt-dlp"),
            None => {
                println!("Video ID not available");
                return Ok(());
            }
        }
    } else {
        println!("No results found.");
        return Ok(());
    };

    let stream_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    play_stream_ffplay(&stream_url)?;
    println!("Stream URL: {}", stream_url);

    Ok(())
}

fn play_stream_ffplay(url: &str) -> std::io::Result<()> {
    Command::new("ffplay")
        .arg("-nodisp") // no video window
        .arg("-autoexit") // exit when done
        .arg(url)
        .spawn()?; // non-blocking
    Ok(())
}
