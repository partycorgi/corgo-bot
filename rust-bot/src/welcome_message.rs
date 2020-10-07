use rand::seq::SliceRandom;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use tracing::error;
use serenity::{
  model::misc::Mentionable,
  utils::MessageBuilder
};

#[derive(Clone, Debug, Deserialize)]
pub struct WelcomeMessage {
    pub before_mention: String,
    pub after_mention: String,
}

fn read_welcome_messages_from_file() -> Vec<WelcomeMessage> {
    let file = File::open("welcome_messages.json")
        .expect("There was no welcome_messages.json found");
    let reader = BufReader::new(file);

    let welcome_messages = serde_json::from_reader(reader)
        .expect("welcome_messages.json wasn't able to be parsed properly");

    welcome_messages
}

pub fn get_welcome_message(new_member: &impl Mentionable) -> String {
    let welcome_messages = read_welcome_messages_from_file();

    let rand_msg = match welcome_messages.choose(&mut rand::thread_rng()) {
        Some(welcome_message) => welcome_message.clone(),
        None => {
          error!(err = "No welcome message was found from the file");
          WelcomeMessage {
            before_mention: String::from("Welcome to the server, "),
            after_mention: String::from(".")
          }
        },
    };

    let welcome_message = MessageBuilder::new()
    .push(&rand_msg.before_mention)
    .mention(new_member)
    .push(&rand_msg.after_mention)
    .build();

    welcome_message
}
