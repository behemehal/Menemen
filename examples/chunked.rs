use menemen::prelude::*;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    let response = request.send().await?;

    println!("Response: {:#?}", response.headers);

    let response_string = response.text().await?;

    let r = "asd";

    {
        println!("{}", r);
        
    }

    print!("asd: {}", r);


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
