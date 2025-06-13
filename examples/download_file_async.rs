use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use menemen::prelude::*;
use std::{
    cmp::min,
    fmt::Write,
    panic,
    time::{Duration, Instant},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

// Convert speed to string in Kbps or Mbps
fn speed_to_string(speed_kbps: f64) -> String {
    if speed_kbps < 1024.0 {
        format!("{:.2} Kbps", speed_kbps)
    } else {
        format!("{:.2} Mbps", speed_kbps / 1024.0)
    }
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://ipv4.download.thinkbroadband.com/1GB.zip",
        RequestTypes::GET,
    )?;

    let mut response = request.send().await?;

    let mut file = File::create("./20MB.zip").await?;
    let mut last_instant = Instant::now();
    let mut collected_byte_len = 0;
    let mut stream_read_len = 0;

    let content_len = match response
        .headers
        .iter_mut()
        .find(|h| h.name == "Content-Length")
    {
        Some(header) => header.value.parse::<usize>().unwrap_or_else(|_| 0),
        None => 0,
    };

    let pb = ProgressBar::new(content_len as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    let mut buffer = [0; 8192 * 2]; // Buffer of 8KB
    loop {
        match response.stream.read(&mut buffer).await {
            Ok(read_bytes) => {
                if read_bytes == 0 {
                    break;
                }

                stream_read_len += read_bytes;
                collected_byte_len += read_bytes;

                let elapsed_duration = last_instant.elapsed();
                if elapsed_duration >= Duration::from_secs(1) {
                    let elapsed_secs = elapsed_duration.as_secs_f64();
                    let speed_kbps = collected_byte_len as f64 / elapsed_secs / 1024.0;
                    collected_byte_len = 0;
                    last_instant = Instant::now();

                    if content_len == 0 {
                        pb.set_style(
                            ProgressStyle::with_template(
                                "{spinner:.green} Downloading without content length [{elapsed_precise}] (Downloaded {bytes}, {bytes_per_sec})",
                            )
                            .unwrap()
                            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                            })
                            .progress_chars("#>-"),
                        );
                        pb.set_position(stream_read_len as u64);
                    } else {
                        let new = min(stream_read_len as u64, content_len as u64);
                        pb.set_message(format!(
                            "Downloading with speed: {}",
                            speed_to_string(speed_kbps)
                        ));
                        pb.set_position(new);
                    }
                }

                file.write_all(&buffer[..read_bytes]).await?;
            }
            Err(e) => {
                panic!("Error reading stream: {}", e);
            }
        };
    }
    pb.finish_with_message("Download complete");
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("Please enable the 'async' feature to run this example.");
}
