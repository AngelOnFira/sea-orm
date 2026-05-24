//! Realistic typed-PK domain code for the snowflake_chat fixture.
//!
//! These functions take typed IDs in their signatures so the compiler
//! catches mixups at every call site. The trybuild fixtures under
//! `tests/value_type_pk_compile_fail/` cover the rejection contract;
//! this module covers the positive side — the functions compile and
//! run when called with the right types.
//!
//! What each function pins:
//!
//! - `send_message`: typed `ChannelId` + `UserId` parameters; the
//!   compiler rejects passing a `GuildId` to either slot.
//! - `find_message_author`: takes a typed `MessageId`, returns the
//!   author's typed `UserId`.
//! - `ban_user_from_guild`: takes a typed `(GuildId, UserId)` composite
//!   key; the compiler rejects passing the components in the wrong
//!   order because `GuildId` and `UserId` are distinct types.
//! - `list_replies_to`: takes a typed `MessageId` and uses it as a
//!   filter value through the column equality API.

use super::{channel, guild, member, message, user};
use sea_orm::{ActiveValue::*, DbErr, DeleteResult, entity::*, query::*};

pub fn send_message<C: ConnectionTrait>(
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
}

pub fn find_message_author<C: ConnectionTrait>(
    db: &C,
    message_id: message::MessageId,
) -> Result<Option<user::UserId>, DbErr> {
    Ok(message::Entity::find_by_id(message_id)
        .one(db)?
        .map(|m| m.author_id))
}

pub fn ban_user_from_guild<C: ConnectionTrait>(
    db: &C,
    guild_id: guild::GuildId,
    user_id: user::UserId,
) -> Result<DeleteResult, DbErr> {
    member::Entity::delete_by_id((guild_id, user_id)).exec(db)
}

pub fn list_replies_to<C: ConnectionTrait>(
    db: &C,
    message_id: message::MessageId,
) -> Result<Vec<message::Model>, DbErr> {
    message::Entity::find()
        .filter(message::Column::ReplyToMessageId.eq(message_id))
        .all(db)
}
