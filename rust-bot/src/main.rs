use std::env;

use libhoney;
use tracing_honeycomb;
use tracing_subscriber::layer::SubscriberExt;
use tracing::{instrument, info, error};

use serenity::{
    framework::standard::macros::{check, command, group},
    framework::standard::{Args, CheckResult, CommandOptions, CommandResult, StandardFramework},
    model::{
        channel::{
            ChannelType, GuildChannel, Message, PermissionOverwrite, PermissionOverwriteType,
        },
        guild::Role,
        id::RoleId,
        permissions::Permissions,
    },
    prelude::*,
    Error,
};

#[group]
#[commands(ping)]
struct General;

#[group]
#[checks(Mod)]
#[commands(create_cohort)]
struct Mod;

#[instrument]
#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
        println!("Error sending message: {}", why);
    }
    return Ok(());
}

// This command provisions out a channel with permissions
// to read/write for users with the corresponding role. It
// will also make the channel read-only for other users.
#[instrument]
#[command]
fn create_cohort(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Spinning up Adventure Club...")?;

    let (cohort_name, channel_name) = gen_names(args.single::<String>().unwrap());
    let role = create_role(ctx, msg, &cohort_name);
    let channel = create_channel(ctx, msg, &channel_name, role);

    match channel {
        Ok(channel) => {
            let reply_msg = format!(
                "Successfully created {}! Feel free to add users.",
                channel.name
            );
            msg.reply(&ctx.http, reply_msg).unwrap();
        }
        Err(_) => {
            msg.reply(&ctx.http, "Failed to create cohort").unwrap();
        }
    };

    Ok(())
}
#[instrument(skip(ctx))]
fn create_role(ctx: &mut Context, msg: &Message, role_name: &str) -> Role {
    let guild = msg.guild(&ctx.cache).unwrap();
    let role = match guild.read().role_by_name(role_name) {
        Some(role) => {
            let content = format!("{} Role already exists!", role.name);
            info!(role_exists=true);
            if let Err(error) = msg.channel_id.say(&ctx.http, content) {
                error!(error.message = ?error);
                println!("{:?}", error);
            };
            role.clone()
        }
        None => {
            info!(role_exists=false);
            guild
            .read()
            .create_role(&ctx, |r| r.name(role_name))
            .unwrap()
        }
    };
    return role;
}

// Permissions for chatting happen here
#[instrument(skip(ctx))]
fn create_channel(
    ctx: &mut Context,
    msg: &Message,
    channel_name: &str,
    role: Role,
) -> Result<GuildChannel, Error> {
    let role_id = role.id;
    let everyone_id = get_everyone_role(ctx, msg).unwrap().id;
    let permission_set = mute_users_without_role_permset(role_id, everyone_id);

    let guild = msg.guild(&ctx.cache).unwrap();
    let new_channel = guild.read().create_channel(&ctx.http, |c| {
        c.name(channel_name)
            .kind(ChannelType::Text)
            .permissions(permission_set)
    });

    new_channel
}
#[derive(Debug)]
struct Handler;

impl EventHandler for Handler {
    #[instrument]
    fn ready(&self, _: Context, _ready_info: serenity::model::gateway::Ready) {
        info!(bot_is_ready = std::time::SystemTime::now());
    }
}
#[instrument]
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

    // When using the framework, we just declare any custom configurations
    // - Adds a prefix to all commands
    // - Grabs all commands in groups
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
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
#[instrument]
#[check]
#[name = "Mod"]
fn mod_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}
#[instrument]
// Generate cohort and channel names
fn gen_names(input_str: String) -> (String, String) {
    let cohort_name = format!("{}", input_str);
    let channel_name = format!("adventure-club: {}", cohort_name);

    (cohort_name, channel_name)
}
#[instrument]
fn mute_users_without_role_permset(
    role_id: RoleId,
    everyone_role_id: RoleId,
) -> Vec<PermissionOverwrite> {
    let mut remove_messaging = Permissions::empty();
    remove_messaging.insert(Permissions::SEND_MESSAGES);
    remove_messaging.insert(Permissions::SEND_TTS_MESSAGES);
    vec![
        PermissionOverwrite {
            allow: Permissions::SEND_MESSAGES,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(role_id),
        },
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: remove_messaging,
            kind: PermissionOverwriteType::Role(everyone_role_id),
        },
    ]
}
#[instrument(skip(ctx))]
fn get_everyone_role(ctx: &mut Context, msg: &Message) -> Option<Role> {
    let guild = msg.guild(&ctx.cache).unwrap();
    for (_, role) in guild.read().roles.iter() {
        if role.name == "@everyone" {
            return Some(role.clone());
        }
    }
    return None;
}
