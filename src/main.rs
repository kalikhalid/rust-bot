use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::ptr::read;
use qrcode::QrCode;
use qrcode::types::QrError;
use rand::{distributions::Alphanumeric, Rng};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
};
use image::Luma;
use teloxide::types::{InputFile, ResponseParameters};
use tokio::fs;
use tokio::fs::OpenOptions;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command{
    #[command(description="start message handler.")]
    Start,
    #[command(description = "handle a username and an age.", parse_with = "split")]
    Create { activations: String, act: u8},
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()>{
    match cmd {
        Command::Start => {
            let message: &str = msg.text().unwrap();
            let msgs: Vec<&str> = message.split(' ').collect();
            if msgs.len() < 2{
                bot.send_message(msg.chat.id, "HI. This is start message.").await?;
                return Ok(());
            }
            let file = File::open("src/codes.txt")?;
            let reader = BufReader::new(file);
            let mut codes: Vec<(String, u32)> = Vec::new();
            for line in reader.lines() {
                let line = line?;
                let mut parts = line.split_whitespace();
                if let (Some(code), Some(count_str), None) = (parts.next(), parts.next(), parts.next()) {
                    if let Ok(count) = count_str.parse::<u32>() {
                        codes.push((code.to_string(), count));
                    }
                }
            }
            if let Some(index) = codes.iter().position(|(code, _)| code == msgs[1]) {
                codes[index].1 -=  1;
                if codes[index].1 ==  0 {
                    codes.remove(index);
                }
                bot.send_message(msg.chat.id, "You owned 20 skillcoins.").await?;
            }else{
                bot.send_message(msg.chat.id, "This code is not avalible").await?;
            }
            let file = File::create("src/codes.txt")?;
            let mut writer = BufWriter::new(file);
            for (code, count) in codes {
                writeln!(writer, "{} {}", code, count)?;
            }

        }
        Command::Create {activations, act} => {
            let mut file = File::create("src/codes.txt")?;
            let code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect();
            writeln!(file, "{} {}", code, activations.trim().parse::<u8>().expect("err"))?;
            let qr = QrCode::new(format!("https://t.me/AppleTeabot?start={}", code)).unwrap();
            let img = qr.render::<Luma<u8>>().build();
            img.save(format!("src/q_{}_code.png", code)).unwrap();
            let qr_send = InputFile::file(format!("src/q_{}_code.png", code));
            bot.send_photo(msg.chat.id, qr_send).await?;
            fs::remove_file(format!("src/q_{}_code.png", code)).await?;
        }
    }
    Ok(())
}
