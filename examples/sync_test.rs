use std::{fs, io::copy};
use menemen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    let mut response = request.send()?;
    let mut file = fs::File::create("./sync20MB.txt")?;

    //let mut buf = Vec::new();
    //response.stream.read_to_end(&mut buf).await?;

    //println!("Buf: {:?}", buf);
    copy(&mut response, &mut file)?;
    //response.stream.copy(&mut file).await?;

    Ok(())
}
