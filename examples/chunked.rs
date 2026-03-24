//! Demonstrates reading a chunked-transfer response in both async and blocking modes.

use menemen::prelude::*;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

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
    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    let mut response = request.send()?;

    println!("Response: {:#?}", response.headers);

    let response_string = response.text()?;

    println!(
        "Response: ({}): {:?}",
        response.response_info.status_code, response_string
    );

    Ok(())
}
