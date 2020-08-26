use std::env;

use libhoney;
use std::collections::HashSet;
use tracing::{error, info, instrument};
use tracing_honeycomb;
use tracing_subscriber::layer::SubscriberExt;

use serenity::{
    framework::standard::macros::{command, group},
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
        StandardFramework,
    },
    model::prelude::{Message, MessageActivityKind, UserId},
    prelude::*,
};

mod commands;
use commands::mod_group::MOD_GROUP;

const CHANNEL__LISTENING_PARTY: u64 = 742445700998103132;

#[group]
#[commands(ping, pin)]
struct General;

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hello! こんにちは！Hola! Bonjour! 您好!\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Strike"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Strike"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).into()
}

#[instrument(skip(ctx))]
#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
        println!("Error sending message: {}", why);
    }
    return Ok(());
}

#[instrument(skip(ctx))]
#[command]
fn pin(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let message_id = args.single::<u64>()?;
    let result = ctx
        .http
        .get_message(*msg.channel_id.as_u64(), message_id)
        .and_then(|rmsg| rmsg.pin(&ctx.http));

    if let Err(e) = result {
        println!("{}", e);
    }
    Ok(())
}

#[derive(Debug)]
struct Handler;

impl EventHandler for Handler {
    #[instrument]
    fn ready(&self, _: Context, _ready_info: serenity::model::gateway::Ready) {
        info!(bot_is_ready = ?std::time::SystemTime::now());
    }
    #[instrument(skip(_ctx, _new_message))]
    fn message(&self, _ctx: Context, _new_message: Message) {
        if _new_message.channel_id == CHANNEL__LISTENING_PARTY {
            match &_new_message.activity {
                Some(activity) => match activity.kind {
                    MessageActivityKind::LISTEN => match _new_message.pin(_ctx.http) {
                        Ok(_) => {}
                        Err(_) => {}
                    },
                    _ => (),
                },
                None => (),
            }
        }
    }
}

#[instrument]
fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    //Sets up the tracing stuff.
    let honeycomb_config = libhoney::Config {
        options: libhoney::client::Options {
            api_key: env::var("HONEYCOMB_API_KEY")
                .expect("expected a honeycomb api key in the environment"),
            dataset: env::var("HONEYCOMB_DATASET_NAME").expect("expected a honeycomb dataset name"),
            ..libhoney::client::Options::default()
        },
        transmission_options: libhoney::transmission::Options::default(),
    };

    let honeycomb_tracing_layer =
        tracing_honeycomb::new_honeycomb_telemetry_layer("honeycomb-service", honeycomb_config);

    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_subscriber::fmt::Layer::default()) //prints logs to console
        .with(honeycomb_tracing_layer); //submits logs to honeycomb.

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting global default tracer failed");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // When using the framework, we just declare any custom configurations
    // - Adds a prefix to all commands
    // - Grabs all commands in groups
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .help(&MY_HELP)
            .group(&GENERAL_GROUP)
            .group(&MOD_GROUP),
    );

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        error!(?why);
        println!("Client error: {:?}", why);
    }
}
