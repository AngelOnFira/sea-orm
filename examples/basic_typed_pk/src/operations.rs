//! Realistic typed-PK domain code over the generated entities.
//!
//! Every function signature carries typed IDs, so the compiler catches
//! mixups at every call site — passing a `GuildId` where a `UserId` is
//! expected, swapping `(GuildId, UserId)` to `(UserId, GuildId)`, etc.
//! The trybuild fixtures under `tests/value_type_pk_compile_fail/`
//! pin the rejection contract; this module is the positive side.
//!
//! `send_message_with_mention` deliberately threads four typed
//! parameters in a row (`ChannelId`, two `UserId`s, `MessageId`) — the
//! kind of API where the type system actually earns its keep.

use crate::entity::{channel, guild, member, message, user};
use sea_orm::{ActiveValue::*, DbErr, DeleteResult, entity::*, query::*};

pub async fn send_message<C: ConnectionTrait>(
    db: &C,
    channel_id: channel::ChannelId,
    author_id: user::UserId,
    content: String,
) -> Result<message::Model, DbErr> {
    message::ActiveModel {
        channel_id: Set(channel_id),
        author_id: Set(author_id),
        mention_user_id: Set(None),
        reply_to_message_id: Set(None),
        content: Set(content),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn send_message_with_mention<C: ConnectionTrait>(
    db: &C,
    channel_id: channel::ChannelId,
    author_id: user::UserId,
    mention: user::UserId,
    reply_to: Option<message::MessageId>,
    content: String,
) -> Result<message::Model, DbErr> {
    message::ActiveModel {
        channel_id: Set(channel_id),
        author_id: Set(author_id),
        mention_user_id: Set(Some(mention)),
        reply_to_message_id: Set(reply_to),
        content: Set(content),
        ..Default::default()
    }
    .insert(db)
    .await
}

pub async fn add_member<C: ConnectionTrait>(
    db: &C,
    guild_id: guild::GuildId,
    user_id: user::UserId,
    nickname: Option<String>,
) -> Result<member::Model, DbErr> {
    member::ActiveModel {
        guild_id: Set(guild_id),
        user_id: Set(user_id),
        nickname: Set(nickname),
    }
    .insert(db)
    .await
}

pub async fn find_member<C: ConnectionTrait>(
    db: &C,
    guild_id: guild::GuildId,
    user_id: user::UserId,
) -> Result<Option<member::Model>, DbErr> {
    member::Entity::find_by_id((guild_id, user_id)).one(db).await
}

pub async fn ban_user_from_guild<C: ConnectionTrait>(
    db: &C,
    guild_id: guild::GuildId,
    user_id: user::UserId,
) -> Result<DeleteResult, DbErr> {
    member::Entity::delete_by_id((guild_id, user_id))
        .exec(db)
        .await
}

pub async fn list_replies_to<C: ConnectionTrait>(
    db: &C,
    message_id: message::MessageId,
) -> Result<Vec<message::Model>, DbErr> {
    message::Entity::find()
        .filter(message::Column::ReplyToMessageId.eq(message_id))
        .all(db)
        .await
}
