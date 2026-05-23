//! Runtime tests for the snowflake_chat fixture.
//!
//! Every PK in this fixture is a `DeriveValueType` newtype wrapping
//! `i64`, declared with no `#[sea_orm(auto_increment)]` annotation.
//! The `PrimaryKeyTrait::auto_increment()` impl emitted by the macro
//! resolves through `PkAutoIncrementHint` to the inner `i64` and back
//! to `true`. If the trait resolution regresses (e.g. someone reverts
//! to a textual heuristic that can't see through the wrapper), schema
//! creation here will stop emitting `AUTOINCREMENT` and the inserts
//! that omit the `id` will fail.

pub mod common;

use common::TestContext;
use common::snowflake_chat::{
    channel, dm_thread, guild, member, message, reaction,
    user::{self, UserId},
};
use sea_orm::{ActiveValue::*, PrimaryKeyTrait, entity::*};
use sea_orm_macros::test;

#[test]
fn auto_increment_resolves_via_trait_for_all_pks() -> Result<(), sea_orm::DbErr> {
    // Pure compile-time + macro-emit assertions. No DB roundtrip needed:
    // if the trait resolution silently flips to `false` for any of these,
    // these assertions fail before any test driver is involved.
    assert!(<guild::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(<user::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(<channel::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(<message::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(<dm_thread::PrimaryKey as PrimaryKeyTrait>::auto_increment());

    // Composite PKs always report false regardless of column types.
    assert!(!<member::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(!<reaction::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    Ok(())
}

#[test]
fn snowflake_chat_end_to_end() -> Result<(), sea_orm::DbErr> {
    let ctx = TestContext::new("snowflake_chat_end_to_end");
    let db = &ctx.db;

    db.get_schema_builder()
        .register(guild::Entity)
        .register(user::Entity)
        .register(channel::Entity)
        .register(member::Entity)
        .register(message::Entity)
        .register(dm_thread::Entity)
        .register(reaction::Entity)
        .apply(db)?;

    // Insert a guild without specifying the PK. If auto_increment didn't
    // resolve, the DB would reject this for missing `id`. `.insert()`
    // returns the persisted `Model`, with `id` already populated by the
    // round-trip.
    let guild = guild::ActiveModel {
        name: Set("Cooks United".to_string()),
        ..Default::default()
    }
    .insert(db)?;
    let guild_id: guild::GuildId = guild.id;

    let alice = user::ActiveModel {
        username: Set("alice".to_string()),
        ..Default::default()
    }
    .insert(db)?;
    let bob = user::ActiveModel {
        username: Set("bob".to_string()),
        ..Default::default()
    }
    .insert(db)?;
    let alice_id: UserId = alice.id;
    let bob_id: UserId = bob.id;

    // Channel FK back to guild uses the typed parent ID — passing a
    // raw `i64` or a different entity's ID would be a type error.
    let general = channel::ActiveModel {
        guild_id: Set(guild_id),
        name: Set("general".to_string()),
        ..Default::default()
    }
    .insert(db)?;

    // Self-ref + multi-FK to user. Author and mention go through their
    // role wrappers so a swap doesn't compile.
    let first = message::ActiveModel {
        channel_id: Set(general.id),
        author_id: Set(message::MessageAuthorId(alice_id)),
        mention_user_id: Set(None),
        reply_to_message_id: Set(None),
        content: Set("hello world".to_string()),
        ..Default::default()
    }
    .insert(db)?;

    let reply = message::ActiveModel {
        channel_id: Set(general.id),
        author_id: Set(message::MessageAuthorId(bob_id)),
        mention_user_id: Set(Some(message::MessageMentionUserId(alice_id))),
        reply_to_message_id: Set(Some(first.id)),
        content: Set("hi alice".to_string()),
        ..Default::default()
    }
    .insert(db)?;

    // Composite-PK lookup: insert a membership row and round-trip
    // `find_by_id((GuildId, UserId))`.
    member::ActiveModel {
        guild_id: Set(guild_id),
        user_id: Set(alice_id),
        nickname: Set(Some("Chef Alice".to_string())),
        ..Default::default()
    }
    .insert(db)?;
    let fetched_member = member::Entity::find_by_id((guild_id, alice_id))
        .one(db)?
        .expect("composite-PK lookup");
    assert_eq!(fetched_member.nickname.as_deref(), Some("Chef Alice"));

    // DM thread: pure role-wrapper case. Each participant column is its
    // own newtype, so accidentally writing the same wrapper for both
    // slots wouldn't compile.
    let dm = dm_thread::ActiveModel {
        participant_a: Set(dm_thread::DmThreadParticipantA(alice_id)),
        participant_b: Set(dm_thread::DmThreadParticipantB(bob_id)),
        ..Default::default()
    }
    .insert(db)?;
    assert_eq!(dm.participant_a.0, alice_id);

    // Reaction: three-column composite PK with two typed components.
    reaction::ActiveModel {
        message_id: Set(reply.id),
        user_id: Set(alice_id),
        emoji: Set(":wave:".to_string()),
        ..Default::default()
    }
    .insert(db)?;
    let rxn = reaction::Entity::find_by_id((reply.id, alice_id, ":wave:".to_string()))
        .one(db)?
        .expect("composite reaction lookup");
    assert_eq!(rxn.emoji, ":wave:");

    // Self-ref: pull the reply chain by querying messages whose
    // reply_to_message_id points at the first message.
    let replies = message::Entity::find()
        .filter(message::Column::ReplyToMessageId.eq(first.id))
        .all(db)?;
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].content, "hi alice");

    Ok(())
}
