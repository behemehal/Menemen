use menemen::request::{ContentTypes, Request, RequestTypes};
use std::{
    fs::File,
    io::Read,
    io::Write,
    panic,
    time::{Instant, SystemTime},
};

//Convert byte size to string
fn byte_size_to_string(size: u64) -> String {
    if size < 1024 {
        return format!("{}B", size);
    } else if size < 1024 * 1024 {
        return format!("{}KB", (size / 1024));
    } else {
        return format!("{}MB", (size / 1024 / 1024));
    }
}

fn main() {
    let mut request = Request::new(
        "ipv4.download.thinkbroadband.com/10MB.zip",
        RequestTypes::GET,
    )
    .unwrap();
    request.set_header("Connection", "close");
    request.content_type = ContentTypes::OctetStream;
    match request.send() {
        Ok(mut e) => {
            let mut file = File::create("./elliec.html").unwrap();

            let mut since_ms = Instant::now();
            let mut speed_kbps = 0;

            loop {
                let mut buffer = [0; 1];
                match e.stream.read(&mut buffer) {
                    Ok(q) => {
                        if q == 0 {
                            break;
                        }
                        //To calculate kbps we should keep track of the time between 1kb
                        if since_ms.elapsed().as_secs() > 1 {
                            speed_kbps = e.stream.read_len / 1024 / since_ms.elapsed().as_secs();
                            since_ms = Instant::now();
                        }

                        if e.stream.content_len == -1 {
                            println!(
                                "Downloading Without Content Len: {}bytes with: {}kbps",
                                byte_size_to_string(e.stream.read_len),
                                byte_size_to_string(speed_kbps)
                            );
                        } else {
                            let percent =
                                e.stream.read_len as f64 / e.stream.content_len as f64 * 100 as f64;
                            println!(
                                "Downloading: {} of {}; {}% {}kbps",
                                byte_size_to_string(e.stream.read_len),
                                byte_size_to_string(e.stream.content_len as u64),
                                percent as u64,
                                speed_kbps
                            )
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
