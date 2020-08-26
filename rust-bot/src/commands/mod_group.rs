use serenity::{
    framework::standard::macros::{check, command, group},
    framework::standard::{Args, CheckResult, CommandOptions, CommandResult},
    model::{
        channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType},
        guild::Role,
        id::RoleId,
        permissions::Permissions,
        prelude::Message,
    },
    prelude::*,
    utils::MessageBuilder,
    Error,
};
use std::env;
use tracing::{error, info, instrument};

#[group]
#[checks(Mod)]
#[commands(create_cohort)]
pub struct Mod;

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
    let role = create_role(ctx, msg, &cohort_name);
    let channel = create_channel(ctx, msg, &channel_name, category_id, &role);

    match channel {
        Ok(channel) => {
            let reply_msg = MessageBuilder::new()
                .push("Successfully created club channel: ")
                .channel(channel)
                .push("! Feel free to add users with the ")
                .role(role)
                .push(" role.")
                .build();

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
            info!(role_exists = true);
            if let Err(error) = msg.channel_id.say(&ctx.http, content) {
                error!(error.message = ?error);
                println!("{:?}", error);
            };
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

    new_channel
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
