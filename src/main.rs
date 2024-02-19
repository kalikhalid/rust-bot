use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::ptr::read;
use qrcode::QrCode;
use rusqlite::{Connection};
use qrcode::types::QrError;
use rand::{distributions::Alphanumeric, Rng};
use teloxide::{
    prelude::*,
    utils::command::BotCommands,
    RequestError,
    dispatching::DefaultKey,
    utils::command::ParseError,
};
use std::sync::Arc;
use image::Luma;
use anyhow::Result;
use teloxide::types::{InputFile, ResponseParameters};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

mod controller;

#[derive(Debug)]
struct Code {
    id: i32,
    code: String,
    activations: u8,
    is_valid: bool,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting command bot...");
    let bot = Bot::from_env();
    Command::repl(bot, handle).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start message handler.")]
    Start,
    #[command(description = "handle a username and an age.", parse_with = "split")]
    Create { activations: String, act: u8 },
}

async fn handle(bot: Bot, msg: Message, cmd: Command) -> Result<()> {
    match cmd {
        Command::Start => {
            let db_file_name = "main.db";
            let connection = Connection::open(db_file_name)?;
            controller::create_table(&connection)?;
            let message: &str = msg.text().unwrap();
            let msgs: Vec<&str> = message.split(' ').collect();
            if msgs.len() < 2 {
                bot.send_message(msg.chat.id, "HI. This is start message.").await?;
            }
            let codes_iter = controller::get_codes(&connection)?;
            for mut code in codes_iter {
                if code.code == msgs[1].to_string() {
                    code.activations -= 1;
                    if code.activations <= 0 {
                        controller::delete_code_by_id(&connection, code.id)?;
                    }
                    bot.send_message(msg.chat.id, "You owned 20 skillcoins.").await?;
                } else {
                    bot.send_message(msg.chat.id, "This code is not avalible").await?;
                }
            }
        }
        Command::Create { activations, act } => {
            let db_file_name = "main.db";
            let connection = Connection::open(db_file_name)?;
            let code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect();
            let activations: u8 = activations.trim().parse::<u8>().expect("err");
            controller::create_code(&connection, &code, activations)?;
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