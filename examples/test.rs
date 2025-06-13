use menemen::request::{Request, RequestTypes};
use tokio::io::{copy, AsyncReadExt};

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    let mut response = request.send().await?;
    let mut file = tokio::fs::File::create("./20MB.txt").await?;

    //let mut buf = Vec::new();
    //response.stream.read_to_end(&mut buf).await?;

    //println!("Buf: {:?}", buf);
    copy(&mut response, &mut file).await?;
    //response.stream.copy(&mut file).await?;

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {}