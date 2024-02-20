use std::fs::File;
use std::io::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Client;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load headers
    let mut file = File::open("params/headers.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let headers: HeaderMap = serde_json::from_str(&contents)?;

    // Create a client
    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    // Load data
    let mut file = File::open("params/data_sample.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: Value = serde_json::from_str(&contents)?;

    // Load params
    let mut file = File::open("params/log.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let params: Value = serde_json::from_str(&contents)?;

    // Make the POST request
    let response = client.post("https://skillbox-kazan.docrm.org/api/AccountApi/Login")
        .json(&params)
        .send()?;

    // Print the cookies
    for cookie in response.cookies() {
        println!("{}", cookie);
    }

    Ok(())
}
