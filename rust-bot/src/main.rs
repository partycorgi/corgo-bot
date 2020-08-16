use std::env;

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

fn create_role(ctx: &mut Context, msg: &Message, role_name: &str) -> Role {
    let guild = msg.guild(&ctx.cache).unwrap();
    let role = match guild.read().role_by_name(role_name) {
        Some(role) => {
            let content = format!("{} Role already exists!", role.name);
            if let Err(error) = msg.channel_id.say(&ctx.http, content) {
                println!("{:?}", error);
            };
            role.clone()
        }
        None => guild
            .read()
            .create_role(&ctx, |r| r.name(role_name))
            .unwrap(),
    };
    return role;
}

// Permissions for chatting happen here
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

struct Handler;

impl EventHandler for Handler {}

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

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
        println!("Client error: {:?}", why);
    }
}

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

// Generate cohort and channel names
fn gen_names(input_str: String) -> (String, String) {
    let cohort_name = format!("{}", input_str);
    let channel_name = format!("adventure-club: {}", cohort_name);

    (cohort_name, channel_name)
}

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

fn get_everyone_role(ctx: &mut Context, msg: &Message) -> Option<Role> {
    let guild = msg.guild(&ctx.cache).unwrap();
    for (_, role) in guild.read().roles.iter() {
        if role.name == "@everyone" {
            return Some(role.clone());
        }
    }
    return None;
}
