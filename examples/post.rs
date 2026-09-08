//! Sends JSON from a local file in both blocking and async variants.

use menemen::request::{ContentTypes, Request, RequestTypes};

#[cfg(not(feature = "async"))]
use std::fs::File;

#[cfg(feature = "async")]
use tokio::fs::File;

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new request.
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST)?;

    // Read a file and append it to the request body.
    let file = File::open("./examples/post.json")?;
    request.append_body(file.into());

    // Set content type.
    request.content_type = ContentTypes::JSON;

    let mut response = request.send()?;

    let text = response.text()?;
    println!("Text: {}", text);
    println!("Response info: {:?}", response.response_info);
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST)?;

    let file = File::open("./examples/post.json").await?;
    request.content_type = ContentTypes::JSON;
    request.append_body(file.into());

    let mut response = request.send().await?;
    let text = response.text().await?;
    println!("Text: {}", text);
    println!("Response info: {:?}", response.response_info);
    Ok(())
}