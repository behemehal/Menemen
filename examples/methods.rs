//! Demonstrates HEAD, PATCH and OPTIONS in both blocking and async modes.
//!
//! - `HEAD` fetches only the headers, which is how you check a resource's size
//!   or existence without downloading it.
//! - `PATCH` sends a partial update.
//! - `OPTIONS` asks which methods a resource allows.
//!
//! Run:
//!
//! ```bash
//! cargo run --example methods
//! ```
//!
//! Set `METHODS_BASE_URL` to point at a different server.

use menemen::request::{Request, RequestTypes};

const DEFAULT_BASE_URL: &str = "http://postman-echo.com";

fn header_of(response: &menemen::response::Response, name: &str) -> String {
    response
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.clone())
        .unwrap_or_else(|| "(absent)".to_string())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("METHODS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

    // HEAD: headers only, no body is transferred.
    let mut head_request = Request::new(&format!("{base_url}/get"), RequestTypes::HEAD)?;
    let mut head_response = head_request.send()?;
    println!("HEAD /get");
    println!("  status:         {}", head_response.response_info.status_code);
    println!("  content-type:   {}", header_of(&head_response, "Content-Type"));
    println!("  content-length: {}", header_of(&head_response, "Content-Length"));
    println!("  body bytes:     {}", head_response.text()?.len());

    // PATCH: a partial update with a JSON body.
    let mut patch_request = Request::new(&format!("{base_url}/patch"), RequestTypes::PATCH)?;
    patch_request.set_header("Content-Type", "application/json");
    patch_request.append_body(String::from("{\"nickname\":\"menemen\"}").into());
    let mut patch_response = patch_request.send()?;
    println!("\nPATCH /patch");
    println!("  status: {}", patch_response.response_info.status_code);
    println!("  echo:   {}", patch_response.text()?);

    // OPTIONS: which methods does this resource allow?
    let mut options_request = Request::new(&format!("{base_url}/get"), RequestTypes::OPTIONS)?;
    let mut options_response = options_request.send()?;
    println!("\nOPTIONS /get");
    println!("  status: {}", options_response.response_info.status_code);
    println!("  allow:  {}", header_of(&options_response, "Allow"));
    let _ = options_response.text();

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("METHODS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

    // HEAD: headers only, no body is transferred.
    let mut head_request = Request::new(&format!("{base_url}/get"), RequestTypes::HEAD)?;
    let mut head_response = head_request.send().await?;
    println!("HEAD /get");
    println!("  status:         {}", head_response.response_info.status_code);
    println!("  content-type:   {}", header_of(&head_response, "Content-Type"));
    println!("  content-length: {}", header_of(&head_response, "Content-Length"));
    println!("  body bytes:     {}", head_response.text().await?.len());

    // PATCH: a partial update with a JSON body.
    let mut patch_request = Request::new(&format!("{base_url}/patch"), RequestTypes::PATCH)?;
    patch_request.set_header("Content-Type", "application/json");
    patch_request.append_body(String::from("{\"nickname\":\"menemen\"}").into());
    let mut patch_response = patch_request.send().await?;
    println!("\nPATCH /patch");
    println!("  status: {}", patch_response.response_info.status_code);
    println!("  echo:   {}", patch_response.text().await?);

    // OPTIONS: which methods does this resource allow?
    let mut options_request = Request::new(&format!("{base_url}/get"), RequestTypes::OPTIONS)?;
    let mut options_response = options_request.send().await?;
    println!("\nOPTIONS /get");
    println!("  status: {}", options_response.response_info.status_code);
    println!("  allow:  {}", header_of(&options_response, "Allow"));
    let _ = options_response.text().await;

    Ok(())
}
