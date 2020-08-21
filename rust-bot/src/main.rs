use std::{
    env,
    str::FromStr
};

use libhoney;
use std::collections::HashSet;
use tracing::{error, info, instrument};
use tracing_honeycomb:: {
    register_dist_tracing_root, TraceId
};
use tracing_subscriber::layer::SubscriberExt;

use serenity::{
    framework::standard::macros::{check, command, group},
    framework::standard::{
        help_commands, macros::help, Args, CheckResult, CommandGroup, CommandOptions,
        CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType},
        guild::Role,
        id::RoleId,
        permissions::Permissions,
        prelude::{Message, UserId},
    },
    prelude::*,
    utils::MessageBuilder,
    Error,
};

#[group]
#[commands(ping, pin)]
struct General;

#[group]
#[checks(Mod)]
#[commands(create_cohort)]
struct Mod;

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
    let _result = register_dist_tracing_root(generate_trace_id_from_message(msg), None);
    if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
        error!(err = ?why);
        println!("Error sending message: {}", why);
    }
    info!("said_pong");
    return Ok(());
}

#[instrument(skip(ctx))]
#[command]
fn pin(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let _result = register_dist_tracing_root(generate_trace_id_from_message(msg), None);
    let message_id = args.single::<u64>()?;
    info!(pinned_message_id = ?message_id);
    let result = ctx
        .http
        .get_message(*msg.channel_id.as_u64(), message_id)
        .and_then(|rmsg| {
            info!(pinned_message = ?rmsg);
            rmsg.pin(&ctx.http)
        });

    if let Err(e) = result {
        error!(err = ?e);
        println!("{}", e);
    }
    
    Ok(())
}

// This command provisions out a channel with permissions
// to read/write for users with the corresponding role. It
// will also make the channel read-only for other users.
#[instrument(skip(ctx))]
#[command]
fn create_cohort(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "Spinning up Adventure Club...")?;

    // Check for adventure club category id
    // You can quickly grab this from "Copy Id"
    // on the category
    let category_id = env::var("A_CLUB_CAT_ID")
        .expect("Double check that you set the adventure club category id properly")
        .parse::<u64>()?;
    let (cohort_name, channel_name) = gen_names(args.single::<String>().unwrap());
    info!(?cohort_name, ?channel_name, channel = tracing::field::Empty, role = tracing::field::Empty, err = tracing::field::Empty);
    let role = create_role(ctx, msg, &cohort_name);
    let channel = create_channel(ctx, msg, &channel_name, category_id, &role);
    info!(?role, ?channel);
    match channel {
        Ok(channel) => {
            let reply_msg = MessageBuilder::new()
                .push("Successfully created club channel: ")
                .channel(channel)
                .push("! Feel free to add users with the ")
                .role(role)
                .push(" role.")
                .build();
            info!(?reply_msg);
            msg.reply(&ctx.http, reply_msg).unwrap();
        }
        Err(err) => {
            msg.reply(&ctx.http, "Failed to create cohort").unwrap();
            info!(?err)
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
            info!(role_exists = true);
            if let Err(error) = msg.channel_id.say(&ctx.http, content) {
                error!(err = ?error);
                println!("{:?}", error);
            };
            info!(role_exists = true, ?role);
            role.clone()
        }
        None => {
            info!(role_exists = false);
            guild
                .read()
                .create_role(&ctx, |r| r.name(role_name).colour(16744330))
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
    category_id: u64,
    role: &Role,
) -> Result<GuildChannel, Error> {
    let role_id = role.id;
    let everyone_id = get_everyone_role(ctx, msg).unwrap().id;
    let permission_set = mute_users_without_role_permset(role_id, everyone_id);

    let guild = msg.guild(&ctx.cache).unwrap();
    let new_channel = guild.read().create_channel(&ctx.http, |c| {
        c.name(channel_name)
            .category(category_id)
            .kind(ChannelType::Text)
            .permissions(permission_set)
    });
    info!(?new_channel);
    new_channel
}
#[derive(Debug)]
struct Handler;

impl EventHandler for Handler {
    #[instrument]
    fn ready(&self, _: Context, _ready_info: serenity::model::gateway::Ready) {
        //generate a trace for the ready command so the ready info can be sent to Honeycomb. 
       let _result = register_dist_tracing_root(TraceId::generate(), None);
        info!(bot_is_ready = ?std::time::SystemTime::now());
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
        error!(err = ?why);
        println!("Client error: {:?}", why);
    }
}
#[instrument(skip(ctx))]
#[check]
#[name = "Mod"]
fn mod_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    // Party Corgi - Mod Role Id = 639531892437286959
    let mod_role_id: RoleId = 639531892437286959.into();
    if let Some(member) = msg.member(&ctx.cache) {
        return member.roles.contains(&mod_role_id).into();
    }

    return false.into();
}
#[instrument]
// Generate cohort and channel names
fn gen_names(input_str: String) -> (String, String) {
    let cohort_name = format!("adventure-club: {}", input_str);
    let channel_name = format!("{}", input_str);

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
// Generates a TraceId from the message.id. If the cast fails, create a random TraceId.
// We do this so that we can have a single trace that spans the length of the message.
// This way we can use the "before" & "after" fns in a single trace.
//Note that we assume that message ids are permanently unique to do this. 
fn generate_trace_id_from_message(msg: &Message) -> TraceId {
    match TraceId::from_str(&msg.id.to_string()) {
        Ok(trace_id) => trace_id,
        Err(err) => {
            // if casting errors, generate a fresh id. 
            error!(message_id = %msg.id, ?err,  "error_converting_message_id_to_trace" );
            TraceId::generate()
        }
    }
}