use std::collections::HashMap;

use menemen::request::{Request, RequestTypes};
use serde::{Deserialize, Serialize};

#[cfg(feature = "async")]
use tokio::io::AsyncReadExt;

//
#[derive(Serialize, Deserialize, Debug)]
struct HttpRequest {
    method: String,
    protocol: String,
    host: String,
    path: String,
    ip: String,
    headers: HashMap<String, String>,
    parsedQueryParams: HashMap<String, String>,
    parsedBody: ParsedBody,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParsedBody {
    textFields: HashMap<String, String>,
    files: Vec<FileField>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FileField {
    name: String,
    fileName: String,
    #[serde(rename = "Content-Disposition")]
    content_disposition: String,
    #[serde(rename = "Content-Type")]
    content_type: String,
}

//

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use menemen::{form_data::FormData, multipart_form::MultipartFormData};
    use tokio::fs::File;

    //let form_data: FormData = vec![("key", "value")].into();

    let mut form_data = MultipartFormData::new();

    //form_data.add_stream("strm", Box::new(file));
    //form_data.add_file("file", "./Cargo.toml").await?;
    //form_data.add_file("allMightyZip", "./20MB.zip").await?;

    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    request.set_header(&"Accept-Encoding", &"gzip");
    request.append_body(form_data.into());

    let mut response = request.send().await?;

    println!("Response: {:#?}", response.headers);

    let response_string = response.text().await?;
    //let response_json = response.json::<HttpRequest>().await?;

    println!(
        "Response: ({}): {:?}",
        response.response_info.status_code, response_string
    );
    //println!("Response: ({:#?}): {:?}", response.response_info.status_code, response_json);
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use menemen::{form_data::FormData, multipart_form::MultipartFormData};
    use tokio::fs::File;

    //let form_data: FormData = vec![("key", "value")].into();

    let mut form_data = MultipartFormData::new();

    //form_data.add_stream("strm", Box::new(file));
    //form_data.add_file("file", "./Cargo.toml").await?;
    //form_data.add_file("allMightyZip", "./20MB.zip").await?;

    let mut request = Request::new(
        "http://anglesharp.azurewebsites.net/Chunked",
        RequestTypes::GET,
    )?;

    //request.set_header(&"Accept-Encoding", &"gzip");
    request.append_body(form_data.into());

    let mut response = request.send()?;

    println!("Response: {:#?}", response.headers);

    let response_string = response.text()?;
    //let response_json = response.json::<HttpRequest>().await?;

    println!(
        "Response: ({}): {:?}",
        response.response_info.status_code, response_string
    );
    //println!("Response: ({:#?}): {:?}", response.response_info.status_code, response_json);
    Ok(())
}
