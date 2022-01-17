use menemen::request::{ContentTypes, Request, RequestTypes};
use std::{
    fs::File,
    io::{self},
    io::{BufRead, Write},
    panic,
    time::Instant,
};

//Convert byte size to string
fn byte_size_to_string(size: usize) -> String {
    if size < 1024 {
        return format!("{}B", size);
    } else if size < 1024 * 1024 {
        return format!("{}KB", (size / 1024));
    } else {
        return format!("{}MB", (size / 1024 / 1024));
    }
}

//Convert seconds to string
fn seconds_to_string(seconds: usize) -> String {
    if seconds < 60 {
        return format!("{}s", seconds);
    } else if seconds < 60 * 60 {
        return format!("{}m", seconds / 60);
    } else {
        return format!("{}h", seconds / 60 / 60);
    }
}

fn main() {
    let mut request = Request::new(
        "ipv4.download.thinkbroadband.com/20MB.zip",
        RequestTypes::GET,
    )
    .unwrap();
    request.set_header("Connection", "close");
    request.content_type = ContentTypes::OctetStream;
    match request.send() {
        Ok(mut e) => {
            let mut file = File::create("./20MB.zip").unwrap();
            let mut since_ms = Instant::now();
            let mut speed_kbps = 0;
            let mut collected_byte_len = 0;
            let mut elapsed_secs = 0;
            let mut stream_read_len = 0;
            let stdout = io::stdout();

            let content_len = match e.headers.iter_mut().find(|h| h.name == "Content-Length") {
                Some(header) => match header.value.parse::<usize>() {
                    Ok(d) => d,
                    Err(_) => 0,
                },
                None => 0,
            };

            loop {
                let mut buffer: Vec<u8> = Vec::new();
                match e.stream.read_until(0, &mut buffer) {
                    Ok(q) => {
                        stream_read_len += buffer.len();
                        if q == 0 {
                            break;
                        }

                        //To calculate kbps we should keep track of the time between 1kb
                        if since_ms.elapsed().as_secs() > 1 {
                            speed_kbps = collected_byte_len;
                            elapsed_secs += 1;
                            collected_byte_len = 0;
                            since_ms = Instant::now();
                        }
                        collected_byte_len += buffer.len();
                        if content_len == 0 {
                            println!(
                                "Downloading Without Content Len: {}bytes with: {}kbps | Active Time: {}s",
                                byte_size_to_string(content_len),
                                byte_size_to_string(speed_kbps) ,
                                elapsed_secs,
                            );
                        } else {
                            let percent = stream_read_len as f64 / content_len as f64 * 100 as f64;

                            let output = format!(
                                "Downloading: {} of {}; {}% {}ps | Active Time: {}s | Estimated : {}\n",
                                byte_size_to_string(stream_read_len),
                                byte_size_to_string(content_len),
                                percent as u64,
                                byte_size_to_string(speed_kbps),
                                elapsed_secs,
                                if speed_kbps == 0 {
                                    "?".to_string()
                                } else {
                                    seconds_to_string(
                                        (content_len - stream_read_len) / speed_kbps
                                    )
                                }
                            );
                            stdout.lock().write_all(output.as_bytes()).unwrap()
                        }
                        file.write(&buffer).unwrap();
                    }
                    Err(e) => {
                        panic!("E {}", e)
                    }
                };
            }
            println!("Download complete");
        }
        Err(e) => {
            println!("Request failed: {:#?}", e);
        }
    }
}
