use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::Arc;
use anyhow::Ok;
use reqwest::cookie::{Cookie, Jar};
use serde_json::Value;
use reqwest::{Client, ClientBuilder};
use reqwest::Url;
use tokio;
use serde_json;
use anyhow::{Result, Error};
use base64::decode;
use rand::distributions::Alphanumeric;
use rand::Rng;

// use serde::{Serialize, Deserialize};

// #[derive(Serialize, Deserialize)]
// struct Query {
//     period: Period,
//     watchPeriod: i32,
//     bindingType: i32,
//     rowType: Option<String>,
//     employeeId: Option<String>,
//     subjectId: Option<String>,
//     pageIx: i32,
// }
//
// #[derive(Serialize, Deserialize)]
// struct Period {
//     from: Instant,
//     to: Instant,
// }
//
// #[derive(Serialize, Deserialize)]
// struct Instant {
//     __instant: i64,
// }

//
// #[tokio::main]
// async fn main() -> Result<()>{
    // let cookie_store: Arc<Jar> = Arc::new(Jar::default());
    // let content = std::fs::read_to_string("params/headers.json").unwrap();
    // let client: Client = Client::builder()
    //     .cookie_provider(cookie_store)
    //     .cookie_store(true)
    //     .build()?;
    // let log_file = BufReader::new(File::open("params/log.json")?);
    // let parse_logs: Value  = serde_json::from_reader(log_file)?;
    // let mut params:[(&str, &str); 2] = [
    //     ("login", parse_logs["login"].as_str().unwrap()),
    //     ("password", parse_logs["password"].as_str().unwrap())
    // ];
    //
    //
    // let login_url = Url::parse_with_params("https://skillbox-kazan.docrm.org/api/AccountApi/Login", params)?;
    // println!("{}", login_url);
    // let res = client.post(login_url)
    //     .body(content.clone())
    //     .send()
    //     .await?;

    ///
//     let url = Url::parse("https://skillbox-kazan.docrm.org/api/PaymentApi/DownloadConversionEventReport")?;
//     let cookies_store = Arc::new(Jar::default());
//     cookies_store.add_cookie_str("Token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpZCI6ImVtcGxveWVlLWVjYWYxMGI2LWEyYzQtNDYyMy05NGQ2LWVjYjBkYzg0ZDg2OCIsImRhdGUiOnsiX19pbnN0YW50IjoxNzA4NjA5MDM4OTQ0fX0.gjGLg8nRBo7e4dFayl7RzDf92BFoifgKaS1MpkZEddQ;", &url);
//     let client = Client::builder()
//         .cookie_store(true)
//         .cookie_provider(cookies_store)
//         .build()?;
//
//     let json_content = std::fs::read_to_string("params/data_sample.json")?;
//     let res = client.post(url)
//         .body(json_content)
//         .send()
//         .await?;
//     let json_data: Value = res.json().await?;
//     // let
//     // let file_contents = json_data["file"]["fileContents"].as_str().ok_or("Failed to extract fileContents").unwrap();
//     let mut file = File::create("file.xlsx")?;
//     let decoded_data = decode(json_data["file"]["fileContents"].as_str().ok_or("Failed to extract fileContents").unwrap())?;
//     file.write_all(&decoded_data)?;
//     Ok(())
// }
pub struct ApiController{
    cookies_store: Arc<Jar>,
    default_headers: String,
}

impl ApiController {
    pub async fn open() -> Result<ApiController, Error>{
        let content = std::fs::read_to_string("api_params/headers.json").unwrap();
        // ("можно ди содать клиента а потом его каждый раз билдить?");
        let client_builder: ClientBuilder = Client::builder();
        let client = client_builder.build()?;
        let log_file = BufReader::new(File::open("api_params/log.json")?);
        let parse_logs: Value  = serde_json::from_reader(log_file)?;
        let params:[(&str, &str); 2] = [
            ("login", parse_logs["login"].as_str().unwrap()),
            ("password", parse_logs["password"].as_str().unwrap())
        ];


        let login_url = Url::parse_with_params("https://skillbox-kazan.docrm.org/api/AccountApi/Login", params)?;
        println!("{}", login_url);
        let res = client.post(login_url.clone())
            .body(content.clone())
            .send()
            .await?;
        let cookie_store: Arc<Jar> = Arc::new(Jar::default());
        let cookies = res.cookies().nth(0).unwrap();
        cookie_store.add_cookie_str(format!("{}={};", cookies.name(), cookies.value()).as_str(), &login_url);
        Ok(ApiController {cookies_store: cookie_store, default_headers: content })
    }
    pub async fn get_table_by_date(&self, days: u32) -> Result<String, Error>{
        let client: Client = Client::builder()
            .cookie_store(true)
            .cookie_provider(self.cookies_store.clone())
            .build()?;
        let url = Url::parse("https://skillbox-kazan.docrm.org/api/PaymentApi/DownloadConversionEventReport")?;

        let json_content = std::fs::read_to_string("api_params/data_sample.json")?;
        let res = client.post(url)
            .body(json_content)
            .send()
            .await?;
        let json_data: Value = res.json().await?;
        let file_contents = json_data["file"]["fileContents"].as_str().ok_or("Failed to extract fileContents").unwrap();
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let file_path = format!("tables/table_{}.xlsx", code);
        let mut file = File::create(file_path.clone())?;
        let decoded_data = decode(json_data["file"]["fileContents"].as_str().ok_or("Failed to extract fileContents").unwrap())?;
        file.write_all(&decoded_data)?;
        Ok(file_path)
    }
}
