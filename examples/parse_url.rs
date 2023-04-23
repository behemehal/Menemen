fn main() {
    match menemen::url::Url::build_from_string("https://example.com".to_string()) {
        Ok(url) => {
            println!("{:#?}", url);
        }
        Err(e) => {
            println!("Failed to parse url: {:?}", e);
        }
    }
}
