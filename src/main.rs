use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}
#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase", description = "These commands are supported:")]
enum Command{
    #[command(description="start message handler.")]
    Start,
    #[command(description="command thant create new qr code.", parse_with = "split")]
    Create,
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
            let contains = reader.lines().any(|line| {
                if let Ok(line) = line {
                    line.contains(msgs[1])
                } else {
                    false
                }
            });
            if contains{
                bot.send_message(msg.chat.id, "You owned 20 skillcoins.").await?;
                return Ok(());
            }
            bot.send_message(msg.chat.id, "This code is not avalible").await?;
        }
        Command::Create => {
            let mut file = File::create("src/codes.txt")?;
            let code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect();
            file.write(&format!("{}\n", code).into_bytes())?;
            let qr = QrCode::new(format!("https://t.me/malik_and_leyla_bot?start={}", code)).unwrap();
            let img = qr.render::<Luma<u8>>().build();
            img.save(format!("src/q_{}_code.png", code)).unwrap();
            let qr_send = InputFile::file(format!("src/q_{}_code.png", code));
            bot.send_photo(msg.chat.id, qr_send).await?;
            fs::remove_file(format!("src/q_{}_code.png", code)).await?;
        }
    }
    Ok(())
}
