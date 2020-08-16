use std::env;

use libhoney;
use tracing_honeycomb;
use tracing_subscriber::layer::SubscriberExt;
use tracing::{instrument, info, error};

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
#[derive(Debug)] //add debug to Handler
struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    
    #[instrument(skip(ctx))] //skip ctx because std debug is not implemented for it
    fn message(&self, ctx: Context, msg: Message) {
        //log the message that gets sent. 
        info!(message=&format!("{:?}", msg)[..]);
        if msg.content == "!ping-corgo-rust" { 
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong from Rust!") {
                //add error to tracing. 
                error!(whyError = &format!("{:?}", why)[..]); 
                println!("Error sending message: {:?}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    #[instrument]
    fn ready(&self, _: Context, ready: Ready) {
        //When ready, log the trace provided by discord & the bot's userId.
        info!(trace = &ready.trace.join(";")[..], userId = &format!("{}?",ready.user.id)[..]);
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    //Sets up the tracing stuff. 
    let honeycomb_config = libhoney::Config {
        options: libhoney::client::Options {
            api_key: env::var("HONEYCOMB_API_KEY").expect("expected a honeycomb api key in the environment"),
            dataset: env::var("HONEYCOMB_DATASET_NAME").expect("expected a honeycomb dataset name"),
            ..libhoney::client::Options::default()
        },
        transmission_options: libhoney::transmission::Options::default(),
    };

    let honeycomb_tracing_layer = tracing_honeycomb::new_honeycomb_telemetry_layer("honeycomb-service", honeycomb_config);

    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(tracing_subscriber::fmt::Layer::default()) //prints logs to console
        .with(honeycomb_tracing_layer); //submits logs to honeycomb. 

    tracing::subscriber::set_global_default(subscriber).expect("setting global default tracer failed");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
