use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use menemen::request::{Request, RequestTypes};

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
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
    let mut response = request.send().await?;

    #[cfg(not(feature = "json"))]
    let response_string = response.text().await?;
    
    #[cfg(feature = "json")]
    let response_json = response.json::<Root>().await?;

    #[cfg(not(feature = "json"))]
    println!("Response: ({:#?}): {}", response.response_info.status_code, response_string);
    #[cfg(feature = "json")]
    println!("Response: ({:#?}): {:?}", response.response_info.status_code, response_json);
    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::POST).unwrap();

    request.body("Hello, World!".into());

    let mut response = request.send()?;
    
    #[cfg(not(feature = "json"))]
    let response_string = response.text()?;
    
    #[cfg(feature = "json")]
    let response_json = response.json::<Root>()?;

    #[cfg(not(feature = "json"))]
    println!("Response: ({:#?}): {}", response.response_info.status_code, response_string);
    
    #[cfg(feature = "json")]
    println!("Response: ({:#?}): {:?}", response.response_info.status_code, response_json);
    Ok(())
}