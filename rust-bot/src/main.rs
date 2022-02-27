use std::{
    collections::HashSet, env, path::Path, str::FromStr,
};

use tracing::{error, info, instrument};

use serenity::{
    async_trait, framework::standard::help_commands,
    model::channel::MessageActivityKind,
};
use serenity::{
    client::{Client, Context, EventHandler},
    http::AttachmentType,
};
use serenity::{
    framework::standard::Args, model::channel::Message,
};
use serenity::{
    framework::standard::{
        macros::{command, group, help},
        CommandGroup, CommandResult, HelpOptions,
        StandardFramework,
    },
    model::id::UserId,
};

mod commands;
use commands::mod_group::MOD_GROUP;

const CHANNEL__LISTENING_PARTY: u64 = 742445700998103132;

#[group]
#[commands(yee_claw, ping, pin)]
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
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    match help_commands::with_embeds(
        context,
        msg,
        args,
        help_options,
        groups,
        owners,
    )
    .await
    {
        Some(_) => CommandResult::Ok(()),
        None => {
            CommandResult::Err("help failed to send".into())
        }
    }
}

#[command]
async fn ping(
    ctx: &Context,
    msg: &Message,
) -> CommandResult {
    if let Err(why) =
        msg.channel_id.say(&ctx.http, "Pong!").await
    {
        error!(err = ?why);
        println!("Error sending message: {}", why);
    }
    info!("said_pong");
    return Ok(());
}

#[command]
async fn pin(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    let message_id = args.single::<u64>()?;
    info!(pinned_message_id = ?message_id);
    let message_result = ctx
        .http
        .get_message(*msg.channel_id.as_u64(), message_id)
        .await?;

    info!(pinned_message = ?message_result);
    let result = message_result.pin(&ctx.http).await;

    if let Err(e) = result {
        error!(err = ?e);
        println!("{}", e);
    }

    Ok(())
}

#[command]
#[aliases("yee-claw")]
async fn yee_claw(
    ctx: &Context,
    msg: &Message,
) -> CommandResult {
    if let Err(why) = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.content("Yeee-claw!").add_file(
                AttachmentType::Path(Path::new(
                    "./yee-claw.png",
                )),
            );
            m
        })
        .await
    {
        println!("Error sending message {}", why);
    }
    Ok(())
}

#[derive(Debug)]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(
        &self,
        _ctx: Context,
        _new_message: Message,
    ) {
        if _new_message.channel_id
            == CHANNEL__LISTENING_PARTY
        {
            match &_new_message.activity {
                Some(activity) => match activity.kind {
                    MessageActivityKind::LISTEN => {
                        match _new_message
                            .pin(_ctx.http)
                            .await
                        {
                            Ok(_) => {}
                            Err(_) => {}
                        }
                    }
                    _ => (),
                },
                None => (),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    // set up tracing
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format::json(),
        )
        .init();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&MOD_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        error!(err = ?why);
        println!("Client error: {:?}", why);
    }
}
