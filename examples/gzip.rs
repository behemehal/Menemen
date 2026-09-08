//! Requests a gzip-encoded response and decodes it via `response.text()`.

use menemen::request::{Request, RequestTypes};

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://behemehal.org", RequestTypes::GET)?;
    request.set_header("Accept-Encoding", "gzip");

    let mut response = request.send()?;

    println!("Response info: {:?}", response.response_info);
    println!("Response headers: {:?}", response.headers);

    let text = response.text()?;
    println!("Text: {}", text);

    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://behemehal.org", RequestTypes::GET)?;
    request.set_header("Accept-Encoding", "gzip");

    let mut response = request.send().await?;

    println!("Response info: {:?}", response.response_info);
    println!("Response headers: {:?}", response.headers);

    let text = response.text().await?;
    println!("Text: {}", text);

    Ok(())
}
