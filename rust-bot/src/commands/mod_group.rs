use std::env;

use tracing::{error, info, instrument};

use serenity::{
    framework::standard::{
        macros::{check, command, group},
        Reason,
    },
    framework::standard::{
        Args, CommandOptions, CommandResult,
    },
    model::{
        channel::{
            ChannelType, GuildChannel, PermissionOverwrite,
            PermissionOverwriteType,
        },
        guild::Role,
        id::RoleId,
        permissions::Permissions,
        prelude::Message,
    },
    prelude::*,
    utils::MessageBuilder,
    Error,
};

#[group]
#[checks(Mod)]
#[commands(add_role_to_users, create_cohort)]
struct Mod;

// Party Corgi - Mod Role Id = 639531892437286959
const MOD_ROLE_ID: u64 = 639531892437286959;

#[command]
#[aliases("add_role", "add-role")]
async fn add_role_to_users(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    if args.len() < 2 {
        error!(
            err =
                "Didn't pass enough arguments to add_role"
        );
        msg.reply(&ctx.http, "You don't have enough args!")
            .await?;
    } else {
        let role_id = args.single::<RoleId>()?;
        let guild = msg.guild(&ctx.cache).await.unwrap();

        for arg in args.iter::<String>() {
            let member_name = &arg?;
            match guild.member_named(member_name) {
                Some(member) => {
                    match guild
                        .member(&ctx.http, member.user.id)
                        .await?
                        .add_role(&ctx.http, &role_id)
                        .await
                    {
                        Ok(_) => {}
                        Err(_) => {
                            let err_msg = format!(
                                    "Role ID {} didn't exist or couldn't be successfully added.",
                                    role_id
                                );
                            error!(err = ?err_msg);
                            msg.reply(&ctx.http, err_msg)
                                .await
                                .unwrap();
                        }
                    };
                }
                None => {
                    let err_msg = format!("Member with the name {} wasn't found", member_name);
                    error!(err = ?err_msg);
                    msg.reply(&ctx.http, err_msg).await?;
                }
            }
        }
    }
    Ok(())
}

// This command provisions out a channel with permissions
// to read/write for users with the corresponding role. It
// will also make the channel read-only for other users.
#[command]
async fn create_cohort(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    msg.channel_id
        .say(
            &ctx.http,
            "Spinning up Adventure Club...",
        )
        .await?;

    // Check for adventure club category id
    // You can quickly grab this from "Copy Id"
    // on the category
    let category_id = env::var("A_CLUB_CAT_ID")
        .expect("Double check that you set the adventure club category id properly")
        .parse::<u64>()?;
    let (cohort_name, channel_name) =
        gen_names(args.single::<String>().unwrap());
    info!(
        ?cohort_name,
        ?channel_name,
        channel = tracing::field::Empty,
        role = tracing::field::Empty,
        err = tracing::field::Empty
    );
    let role = create_role(ctx, msg, &cohort_name).await;
    let channel = create_channel(
        ctx,
        msg,
        &channel_name,
        category_id,
        &role,
    )
    .await;
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
            msg.reply(&ctx.http, reply_msg).await.unwrap();
        }
        Err(err) => {
            msg.reply(&ctx.http, "Failed to create cohort")
                .await
                .unwrap();
            info!(?err)
        }
    };

    Ok(())
}

async fn create_role(
    ctx: &Context,
    msg: &Message,
    role_name: &str,
) -> Role {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let role = match guild.role_by_name(role_name) {
        Some(role) => {
            let content = format!(
                "{} Role already exists!",
                role.name
            );
            info!(role_exists = true);
            if let Err(error) =
                msg.channel_id.say(&ctx.http, content).await
            {
                error!(err = ?error);
                println!("{:?}", error);
            };
            info!(role_exists = true, ?role);
            role.clone()
        }
        None => {
            info!(role_exists = false);
            guild
                .create_role(&ctx, |r| {
                    r.name(role_name).colour(16744330)
                })
                .await
                .unwrap()
        }
    };
    return role;
}

// Permissions for chatting happen here
#[instrument(skip(ctx))]
async fn create_channel(
    ctx: &Context,
    msg: &Message,
    channel_name: &str,
    category_id: u64,
    role: &Role,
) -> Result<GuildChannel, Error> {
    let role_id = role.id;
    let everyone_id =
        get_everyone_role(ctx, msg).await.unwrap().id;
    let permission_set = mute_users_without_role_permset(
        role_id,
        everyone_id,
    );

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let new_channel = guild
        .create_channel(&ctx.http, |c| {
            c.name(channel_name)
                .category(category_id)
                .kind(ChannelType::Text)
                .permissions(permission_set)
        })
        .await;
    info!(?new_channel);
    new_channel
}

#[check]
#[name = "Mod"]
async fn mod_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let mod_role_id: RoleId = MOD_ROLE_ID.into();
    match &msg.member {
        Some(member) => {
            match member.roles.contains(&mod_role_id) {
                true => Ok(()),
                false => Err(Reason::Log(
                    "user does not have the mod role"
                        .to_string(),
                )),
            }
        }
        None => Err(Reason::Log(
            "member doesn't exist".to_string(),
        )),
    }
}
#[instrument]
// Generate cohort and channel names
fn gen_names(input_str: String) -> (String, String) {
    let cohort_name =
        format!("adventure-club: {}", input_str);
    let channel_name = format!("{}", input_str);

    (cohort_name, channel_name)
}
#[instrument]
fn mute_users_without_role_permset(
    role_id: RoleId,
    everyone_role_id: RoleId,
) -> Vec<PermissionOverwrite> {
    let mod_role_id: RoleId = MOD_ROLE_ID.into();
    let mut messaging_perms = Permissions::empty();
    messaging_perms.insert(Permissions::SEND_MESSAGES);
    messaging_perms.insert(Permissions::SEND_TTS_MESSAGES);
    vec![
        PermissionOverwrite {
            allow: Permissions::SEND_MESSAGES,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(role_id),
        },
        PermissionOverwrite {
            allow: messaging_perms,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(
                mod_role_id,
            ),
        },
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: messaging_perms,
            kind: PermissionOverwriteType::Role(
                everyone_role_id,
            ),
        },
    ]
}
#[instrument(skip(ctx))]
async fn get_everyone_role(
    ctx: &Context,
    msg: &Message,
) -> Option<Role> {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    for (_, role) in guild.roles.iter() {
        if role.name == "@everyone" {
            return Some(role.clone());
        }
    }
    return None;
}
