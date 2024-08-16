#![cfg(feature = "async")]

use menemen::request::{ContentTypes, Request, RequestTypes};
use std::{
    panic,
    time::{Duration, Instant},
};
use tokio::fs;

// Convert byte size to string
fn byte_size_to_string(size: usize) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{}KB", size / 1024)
    } else {
        format!("{}MB", size / 1024 / 1024)
    }
}

// Convert speed to string in Kbps or Mbps
fn speed_to_string(speed_kbps: f64) -> String {
    if speed_kbps < 1024.0 {
        format!("{:.2} Kbps", speed_kbps)
    } else {
        format!("{:.2} Mbps", speed_kbps / 1024.0)
    }
}

#[cfg(not(feature = "async"))]
panic!("This example requires async feature to be enabled");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://ipv4.download.thinkbroadband.com/1GB.zip",
        RequestTypes::GET,
    )
    .unwrap();
    request.set_header("Connection", "close");
    request.content_type = ContentTypes::OctetStream;

    let response = request.send().await?;

    let mut file = fs::File("./20MB.zip").unwrap();
    let mut since_ms = Instant::now();
    let mut last_instant = Instant::now();
    let mut collected_byte_len = 0;
    let mut stream_read_len = 0;
    let stdout = io::stdout();

    let content_len = match response
        .headers
        .iter_mut()
        .find(|h| h.name == "Content-Length")
    {
        Some(header) => header.value.parse::<usize>().unwrap_or_else(|_| 0),
        None => 0,
    };

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
                        println!(
                            "Downloading Without Content Len: {} | Speed: {} | Active Time: {}s",
                            byte_size_to_string(stream_read_len),
                            speed_to_string(speed_kbps),
                            since_ms.elapsed().as_secs(),
                        );
                    } else {
                        let percent = (stream_read_len as f64 / content_len as f64) * 100.0;

                        let output = format!(
                            "\rDownloading: {} of {}; {:.2}% | Speed: {} | Active Time: {}s",
                            byte_size_to_string(stream_read_len),
                            byte_size_to_string(content_len),
                            percent,
                            speed_to_string(speed_kbps),
                            since_ms.elapsed().as_secs(),
                        );
                        stdout.lock().write_all(output.as_bytes()).unwrap();
                        stdout.lock().flush().unwrap();
                    }
                }

                file.write_all(&buffer[..read_bytes]).unwrap();
            }
            Err(e) => {
                panic!("Error reading stream: {}", e);
            }
        };
    }
    println!("\nDownload complete");
}
