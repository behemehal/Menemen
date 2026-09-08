//! Demonstrates reading a chunked-transfer response in both async and blocking modes.
//!
//! Expects the bundled demo server to be running:
//!
//! ```bash
//! node examples/chunked_server.js
//! ```
//!
//! Set `CHUNKED_URL` to point at a different endpoint.

use menemen::prelude::*;

const DEFAULT_URL: &str = "http://localhost:3001/chunked";

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("CHUNKED_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());

    println!("Requesting chunked response from: {}", url);
    println!("Tip: run `node examples/chunked_server.js` first.\n");

    let mut request = Request::new(&url, RequestTypes::GET)?;

    let mut response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Request failed: {error}");
            eprintln!("Is the demo server running? `node examples/chunked_server.js`");
            return Ok(());
        }
    };

    println!("Response: {:#?}", response.headers);

    let response_string = response.text().await?;

    println!(
        "Response: ({}): {} bytes",
        response.response_info.status_code,
        response_string.len()
    );
    println!("Body:\n{}", response_string);

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("CHUNKED_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());

    println!("Requesting chunked response from: {}", url);
    println!("Tip: run `node examples/chunked_server.js` first.\n");

    let mut request = Request::new(&url, RequestTypes::GET)?;

    let mut response = match request.send() {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Request failed: {error}");
            eprintln!("Is the demo server running? `node examples/chunked_server.js`");
            return Ok(());
        }
    };

    println!("Response: {:#?}", response.headers);

    let response_string = response.text()?;

    println!(
        "Response: ({}): {} bytes",
        response.response_info.status_code,
        response_string.len()
    );
    println!("Body:\n{}", response_string);

    Ok(())
}
