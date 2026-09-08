//! Sends a JSON POST request and prints response headers and body.

use menemen::prelude::*;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("https://postman-echo.com/POST", RequestTypes::POST)?;

    request.append_body("{\"key\":\"value\"}".into());
    request.content_type = ContentTypes::JSON;

    let mut response = request.send().await?;
    println!("Response: {:#?}", response.headers);

    let response_string = response.text().await?;
    println!(
        "Response: ({}): {:?}",
        response.response_info.status_code, response_string
    );
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("https://postman-echo.com/post", RequestTypes::POST)?;

    request.append_body("{\"key\":\"value\"}".into());
    request.content_type = ContentTypes::JSON;

    let mut response = request.send()?;
    println!("Response: {:#?}", response.headers);

    let response_string = response.text()?;
    println!(
        "Response: ({}): {:?}",
        response.response_info.status_code, response_string
    );
    Ok(())
}
