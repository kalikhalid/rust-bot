use std::ffi::c_char;
use std::fs;
use crate::*;
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;
use teloxide::{Bot, dispatching::DefaultKey, dptree, utils::command::{BotCommands, ParseError}};
use std::ptr::read;
use qrcode::QrCode;
use rusqlite::{Connection};
use qrcode::types::QrError;
use rand::{distributions::Alphanumeric, Rng};
use image::Luma;
use log::warn;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::prelude::Requester;
use teloxide::prelude::Update;
use teloxide::types::{InputFile, Message, Recipient};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "start message handler.")]
    Start,
    #[command(description = "handle a username and an age.", parse_with = "split")]
    Create { activations: String, time: u8 },
    #[command(description = "get period xlsx file ")]
    GetTable { period: u8 },
}


pub struct MyBot {
    pub dispatcher: Dispatcher<Arc<Bot>, anyhow::Error, DefaultKey>,
    pub tg: Arc<Bot>,
}

impl MyBot {
    pub async fn new(config: Arc<config::Config>) -> Result<Self> {
        let tg = Arc::new(Bot::new(config.telegram_bot_token.expose_secret()));
        tg.set_my_commands(Command::bot_commands()).await?;
        let handler = Update::filter_message().branch(
            dptree::filter(|msg: Message, config: Arc<config::Config>| {
                true
            })
                .filter_command::<Command>()
                .endpoint(handle_command),
        );
        let dispatcher = Dispatcher::builder(tg.clone(), handler)
            .dependencies(dptree::deps![config.clone()])
            .default_handler(|upd| async move {
                warn!("unhandled update: {:?}", upd);
            })
            .error_handler(LoggingErrorHandler::with_custom_text(
                "an error has occurred in the dispatcher",
            ))
            .build();

        let my_bot = MyBot {
            dispatcher,
            tg: tg.clone(),
        };
        Ok(my_bot)
    }

    pub fn spawn(
        mut self,
    ) -> (
        tokio::task::JoinHandle<()>,
        teloxide::dispatching::ShutdownToken,
    ) {
        let shutdown_token = self.dispatcher.shutdown_token();
        (
            tokio::spawn(async move { self.dispatcher.dispatch().await }),
            shutdown_token,
        )
    }
}

pub async fn handle_command(
    msg: Message,
    bot: Arc<Bot>,
    command: Command,
    config: Arc<config::Config>,
) -> Result<()> {
    match command {
        Command::Start => {
            let controller = controller::Controller::open(&config)?;
            let message: &str = msg.text().unwrap();
            let msgs: Vec<&str> = message.split(' ').collect();
            if msgs.len() < 2 {
                bot.send_message(msg.chat.id, "HI. This is start message.").await?;
                return Ok(());
            }
            let codes_iter = controller.get_codes()?;
            let mut flag = false;
            for mut code in codes_iter {
                if code.code == msgs[1].to_string() {
                    code.activations -= 1;
                    controller.update_code(code.id)?;
                    if code.activations <= 0 {
                        controller.delete_code_by_id(code.id)?;
                    }
                    flag = true;
                    bot.send_message(msg.chat.id, "You owned 20 skillcoins.").await?;
                }
            }
            if !flag{
                bot.send_message(msg.chat.id, "Code is not avalible.").await?;
            }
        }
        Command::Create { activations, time} => {
            let controller = controller::Controller::open(&config)?;
            let code: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(8)
                .map(char::from)
                .collect();
            let activations: u8 = activations.trim().parse::<u8>()?;
            controller.create_code(&code, activations)?;
            let qr = QrCode::new(format!("https://t.me/AppleTeabot?start={}", code))?;
            let img = qr.render::<Luma<u8>>().build();
            img.save(format!("image_data/q_{}_code.png", code))?;
            let qr_send = InputFile::file(format!("image_data/q_{}_code.png", code));
            bot.send_photo(msg.chat.id, qr_send).await?;
            fs::remove_file(format!("image_data/q_{}_code.png", code))?;
        }
        Command::GetTable {period} => {
            let api_controller = api::ApiController::open().await?;
            let file_path = api_controller.get_table_by_date(30).await?;
            let file = InputFile::file(file_path);
            bot.send_document(msg.chat.id, file).await?;
        }

    }
    Ok(())
}
