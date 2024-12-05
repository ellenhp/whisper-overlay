use std::time::Duration;

use audio::record_wav;
use clap::Parser;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use tempfile::TempDir;
use tokio::time::sleep;
use whisper::read_audio;

mod cli;
mod audio;
mod whisper;

use ct2rs::Whisper;


#[derive(Parser)]
struct Args {
    #[arg(long)]
    model: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let handle = tokio::spawn(async move {
        let whisper = Whisper::new(args.model, Default::default()).expect("Failed to load model.");

        let wav_path = rx.recv().await.expect("Failed to receive wav path");

        println!("Transcribing audio...");
        let samples = read_audio(&wav_path, whisper.sampling_rate()).expect("Failed to read wav");
    
        let res = whisper.generate(&samples, Some("en"), false, &Default::default()).expect("Failed to transcribe");
        let res: Vec<String> = res.iter().map(|s| s.trim().to_string()).collect();
        let text = res.join(" ");
        println!("{}", text);
        let mut enigo = Enigo::new(&Settings { mac_delay: 10, linux_delay: 10, release_keys_when_dropped: true, ..Default::default() }).unwrap();
       
        for c in text.chars() {
            if let Err(e) = enigo.key(Key::Unicode(c), Direction::Click) {
                eprintln!("Failed to type text: {e}")
            }
            sleep(Duration::from_millis(2)).await;
        }
       
        println!("Sent text input");
        });

    let dir = TempDir::new().expect("Failed to create temp dir");
    let wav_path = dir.path().join("recording.wav");
    record_wav(&wav_path).expect("Failed to record wav");

    tx.send(wav_path).await.expect("Failed to send wav path");
    handle.await.unwrap();


}
