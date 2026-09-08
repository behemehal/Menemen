//! Simulates long polling with repeated requests and timeout/backoff handling.

use menemen::request::{Request, RequestTypes};

const DEFAULT_BASE_URL: &str = "http://postman-echo.com/get";
const POLL_ROUNDS: usize = 8;
const CLIENT_WAIT_MS: u64 = 2_000;
const REQUEST_TIMEOUT_MS: u64 = 25_000;

#[cfg(not(feature = "async"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{thread, time::Duration};

    // Example endpoint can be replaced with your own long-poll endpoint.
    let base_url = std::env::var("LONG_POLL_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

    println!("Long polling demo started against: {}", base_url);
    println!("Tip: set LONG_POLL_URL to your own endpoint for real updates.\n");

    let mut last_payload = String::new();
    let mut cursor = 0usize;

    for round in 1..=POLL_ROUNDS {
        let poll_url = format!("{}?channel=chat&cursor={}", base_url, cursor);

        let mut request = Request::new(&poll_url, RequestTypes::GET)?;
        request.set_timeout(REQUEST_TIMEOUT_MS);

        match request.send() {
            Ok(mut response) => {
                let payload = response.text()?;
                if payload != last_payload {
                    println!(
                        "[round {round}] update received (status={})",
                        response.response_info.status_code
                    );
                    println!("{}\n", payload);
                    last_payload = payload;
                    cursor += 1;
                } else {
                    println!("[round {round}] no update");
                }
            }
            Err(error) => {
                println!("[round {round}] request error: {error}");
                println!("retrying after short backoff...\n");
                thread::sleep(Duration::from_millis(1_000));
                continue;
            }
        }

        // In real long-poll clients the server often holds this request open.
        // We also keep a small client-side pause to avoid tight loops.
        thread::sleep(Duration::from_millis(CLIENT_WAIT_MS));
    }

    println!("Long polling demo finished.");
    Ok(())
}

#[cfg(feature = "async")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::time::{sleep, Duration};

    let base_url = std::env::var("LONG_POLL_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

    println!("Long polling demo started against: {}", base_url);
    println!("Tip: set LONG_POLL_URL to your own endpoint for real updates.\n");

    let mut last_payload = String::new();
    let mut cursor = 0usize;

    for round in 1..=POLL_ROUNDS {
        let poll_url = format!("{}?channel=chat&cursor={}", base_url, cursor);

        let mut request = Request::new(&poll_url, RequestTypes::GET)?;
        request.set_timeout(REQUEST_TIMEOUT_MS);

        match request.send().await {
            Ok(mut response) => {
                let payload = response.text().await?;
                if payload != last_payload {
                    println!(
                        "[round {round}] update received (status={})",
                        response.response_info.status_code
                    );
                    println!("{}\n", payload);
                    last_payload = payload;
                    cursor += 1;
                } else {
                    println!("[round {round}] no update");
                }
            }
            Err(error) => {
                println!("[round {round}] request error: {error}");
                println!("retrying after short backoff...\n");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }
        }

        sleep(Duration::from_millis(CLIENT_WAIT_MS)).await;
    }

    println!("Long polling demo finished.");
    Ok(())
}
