use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use anyhow::Ok;
use reqwest::cookie::Jar;
use serde_json::Value;
use reqwest::Client;
use reqwest::Url;
use tokio;
use serde_json;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()>{
    let cookie_store: Arc<Jar> = Arc::new(Jar::default());
    let content = std::fs::read_to_string("params/headers.json").unwrap();
    let client: Client = Client::builder()
        .cookie_provider(cookie_store)
        .cookie_store(true)
        .build()?;
    let log_file = BufReader::new(File::open("params/log.json")?);
    let parse_logs: Value  = serde_json::from_reader(log_file)?;
    let mut params:[(&str, &str); 2] = [
        ("login", parse_logs["login"].as_str().unwrap()),
        ("password", parse_logs["password"].as_str().unwrap())
    ];


    let url = "https://skillbox-kazan.docrm.org/api/AccountApi/Login";
    let url: Url = "https://skillbox-kazan.docrm.org/api/AccountApi/Login?login=79279500595&password=123456".parse().unwrap(); //Url::parse_with_params(url, &params)?;
    println!("{}", url);
    let res = client.post(url)
        .form(&params)
        .body(content)
        .send()
        .await?;
    println!("{}", res.status());
    let nwe_url = "https://skillbox-kazan.docrm.org".parse::<Url>().unwrap();
    let headers = res.cookies();
    for i in headers{
        println!("{:?}", i);
    }
    println!("{:?}", client);
    Ok(())
}

pub struct ApiController;

// impl ApiController {
//     pub fn
// }