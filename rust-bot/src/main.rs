use std::{dbg, env};

use serenity::{
    framework::standard::macros::{command, group, check},
    framework::standard::{
         Args, CheckResult, StandardFramework, CommandOptions, CommandResult},
    model::{channel::GuildChannel, channel::Message, channel::ChannelType},
    prelude::*,
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
// to read/write for users with a role that matches the
// channel.
#[command]
fn create_cohort(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let cohort_name = args.single::<String>().unwrap();
    create_role(ctx, msg, &cohort_name);
    create_channel(ctx, msg, &cohort_name);
    
    Ok(())
}

fn create_role(ctx: &mut Context, msg: &Message, role_name: &str) {
    let start_up_msg = "Spinning up Adventure Club...";
    if let Err(error) = msg.channel_id.say(&ctx.http, start_up_msg) {
        println!("Error sending message: {:?}", error);
    };

    if let Some(guild) = msg.guild(&ctx.cache) {
        match guild.read().role_by_name(role_name) {
            Some(role) => {
                let content = format!("{} Role already exists!", role.name);
                if let Err(error) = msg.channel_id.say(&ctx.http, content) {
                    println!("{:?}", error);
                };
            },
            None => {
                guild.read().create_role(&ctx, |r| r.name(role_name)).unwrap();
            }
        };
    };
}

fn create_channel(ctx: &mut Context, msg: &Message, cohort_name: &str) {
    let channel_name = format!("adventure-club: {}", cohort_name);
    if let Some(guild) = msg.guild(&ctx.cache) {

        match guild.read().create_channel(&ctx, |c| c.name(channel_name)
            .kind(ChannelType::Text)
        ) {
            Ok(channel) => {
                println!("Created channel");
            },
            Err(err) => {
                println!("Channel already exists");
            }
        }
    }
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
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP)
        .group(&MOD_GROUP)
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