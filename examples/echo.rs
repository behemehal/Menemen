use menemen::request::{Request, RequestTypes};
use serde::{Deserialize, Serialize};

#[cfg(feature = "async")]
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
struct Headers {
    pub host: String,
    #[serde(rename = "x-forwarded-proto")]
    pub x_forwarded_proto: String,
    #[serde(rename = "x-request-start")]
    pub x_request_start: String,
    pub connection: String,
    #[serde(rename = "x-forwarded-port")]
    pub x_forwarded_port: String,
    #[serde(rename = "x-amzn-trace-id")]
    pub x_amzn_trace_id: String,
    #[serde(rename = "cache-control")]
    pub cache_control: String,
    #[serde(rename = "user-agent")]
    pub user_agent: String,
    #[serde(rename = "content-type")]
    pub content_type: String,
}

#[derive(Serialize, Deserialize)]
struct Args {}

#[derive(Serialize, Deserialize)]
struct Root {
    pub args: Args,
    pub headers: Headers,
    pub url: String,
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use menemen::{form_data::FormData, multipart_form::MultipartFormData};

    //let form_data: FormData = vec![("key", "value")].into();

    let mut form_data = MultipartFormData::new();

    form_data.add_string("key", "value".into());
    form_data.add_file("file", "./Cargo.toml").await?;

    //.add_string("key", "value".into())
    //.add_file("file", "./Cargo.toml")
    //.await?;

    let mut request = Request::new("http://postman-echo.com/post", RequestTypes::POST)?;
    request.content_type = menemen::request::ContentTypes::FormData;

    request.append_body(form_data.into());

    let mut response = request.send().await?;
    println!("Response: {:#?}", response.response_info);

    let response_string = response.text().await?;
    //let response_json = response.json::<Root>().await?;

    println!(
        "Response: ({:#?}): {}",
        response.response_info.status_code, response_string
    );
    //println!("Response: ({:#?}): {:?}", response.response_info.status_code, response_json);
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/post", RequestTypes::POST).unwrap();
    /*
    let form_data: FormData = vec![("key", "value")].into();
    request.content_type = menemen::request::ContentTypes::FormData;

    request.append_body(form_data.into()); */

    let mut response = request.send()?;

    println!("Response: {:#?}", response.response_info);

    //let response_string = response.text()?;
    //#[cfg(feature = "json")]
    //let response = response.json::<Root>()?;

    //#[cfg(not(feature = "json"))]
    let response = response.text()?;

    println!("Response: {}", response);
    Ok(())
}
