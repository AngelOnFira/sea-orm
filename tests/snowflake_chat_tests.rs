//! Runtime tests for the snowflake_chat fixture.
//!
//! Each PK in this fixture is a `DeriveValueType` newtype wrapping `i64`
//! and is declared with no explicit `#[sea_orm(auto_increment)]`
//! annotation. The pinned contract is:
//!
//!   `PrimaryKeyTrait::auto_increment()` for such a wrapper resolves to
//!   `true` via `PkAutoIncrementHint` delegating through the inner `i64`.
//!
//! Each `insert(...)` below relies on that — the rows omit `id` and the
//! database fills it in. If the trait resolution stops returning `true`
//! for these wrappers, schema creation drops `AUTOINCREMENT` and the
//! inserts fail at runtime.

pub mod common;

use common::TestContext;
use common::snowflake_chat::{
    channel, dm_thread, guild, member, message, reaction,
    user::{self, UserId},
};
use sea_orm::{ActiveValue::*, PrimaryKeyTrait, entity::*, query::*};

#[sea_orm_macros::test]
async fn auto_increment_resolves_via_trait_for_all_pks() -> Result<(), sea_orm::DbErr> {
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

#[sea_orm_macros::test]
async fn snowflake_chat_end_to_end() -> Result<(), sea_orm::DbErr> {
    let ctx = TestContext::new("snowflake_chat_end_to_end").await;
    let db = &ctx.db;

    db.get_schema_builder()
        .register(guild::Entity)
        .register(user::Entity)
        .register(channel::Entity)
        .register(member::Entity)
        .register(message::Entity)
        .register(dm_thread::Entity)
        .register(reaction::Entity)
        .apply(db)
        .await?;

    // Insert a guild without specifying the PK. If auto_increment didn't
    // resolve, the DB would reject this for missing `id`. `.insert()`
    // returns the persisted `Model`, with `id` already populated by the
    // round-trip.
    let guild = guild::ActiveModel {
        name: Set("Cooks United".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let guild_id: guild::GuildId = guild.id;

    let alice = user::ActiveModel {
        username: Set("alice".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let bob = user::ActiveModel {
        username: Set("bob".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let alice_id: UserId = alice.id;
    let bob_id: UserId = bob.id;

    // Channel FK back to guild uses the typed parent ID — passing a
    // raw `i64` or a different entity's ID would be a type error.
    let general = channel::ActiveModel {
        guild_id: Set(guild_id),
        name: Set("general".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Self-ref + multi-FK to user. Author and mention share the parent
    // UserId type (codegen doesn't role-wrap non-PK FK columns), but
    // the surrounding signatures still reject GuildId / ChannelId.
    let first = message::ActiveModel {
        channel_id: Set(general.id),
        author_id: Set(alice_id),
        mention_user_id: Set(None),
        reply_to_message_id: Set(None),
        content: Set("hello world".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let reply = message::ActiveModel {
        channel_id: Set(general.id),
        author_id: Set(bob_id),
        mention_user_id: Set(Some(alice_id)),
        reply_to_message_id: Set(Some(first.id)),
        content: Set("hi alice".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Composite-PK lookup: insert a membership row and round-trip
    // `find_by_id((GuildId, UserId))`.
    member::ActiveModel {
        guild_id: Set(guild_id),
        user_id: Set(alice_id),
        nickname: Set(Some("Chef Alice".to_string())),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let fetched_member = member::Entity::find_by_id((guild_id, alice_id))
        .one(db)
        .await?
        .expect("composite-PK lookup");
    assert_eq!(fetched_member.nickname.as_deref(), Some("Chef Alice"));

    // DM thread: both participants share `UserId`. The function-level
    // safety contract for "this argument is participant A, not B" lives
    // in domain code (see `operations::send_message` for the canonical
    // pattern). Inserting directly here passes typed user IDs through.
    let dm = dm_thread::ActiveModel {
        participant_a: Set(alice_id),
        participant_b: Set(bob_id),
        ..Default::default()
    }
    .insert(db)
    .await?;
    assert_eq!(dm.participant_a, alice_id);

    // Reaction: three-column composite PK with two typed components.
    reaction::ActiveModel {
        message_id: Set(reply.id),
        user_id: Set(alice_id),
        emoji: Set(":wave:".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let rxn = reaction::Entity::find_by_id((reply.id, alice_id, ":wave:".to_string()))
        .one(db)
        .await?
        .expect("composite reaction lookup");
    assert_eq!(rxn.emoji, ":wave:");

    // Self-ref: pull the reply chain by querying messages whose
    // reply_to_message_id points at the first message.
    let replies = message::Entity::find()
        .filter(message::Column::ReplyToMessageId.eq(first.id))
        .all(db)
        .await?;
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].content, "hi alice");

    // Pin coverage of the three PK-taking signatures with a typed PK:
    //   - `find_by_id(TypedPk)`   (already exercised above for composite PKs)
    //   - `Select::filter_by_id(TypedPk)`
    //   - `delete_by_id(TypedPk)`
    // The untyped (raw scalar) paths for the same three are exercised in
    // `tests/query_tests.rs`, `tests/active_model_ex_tests.rs`,
    // `tests/multi_select_tests.rs`, and `tests/delete_by_id_tests.rs`.
    let fetched_via_filter_by_id = guild::Entity::load()
        .filter_by_id(guild_id)
        .one(db)
        .await?
        .expect("filter_by_id with typed PK");
    assert_eq!(fetched_via_filter_by_id.id, guild_id);

    let delete_target = guild::ActiveModel {
        name: Set("To Be Removed".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    let delete_res = guild::Entity::delete_by_id(delete_target.id).exec(db).await?;
    assert_eq!(delete_res.rows_affected, 1);
    assert!(
        guild::Entity::find_by_id(delete_target.id)
            .one(db)
            .await?
            .is_none()
    );

    // Exercise typed-PK domain code from `snowflake_chat::operations`.
    // These function signatures take typed `ChannelId` / `UserId` /
    // `GuildId` / `MessageId` arguments, so swapping the wrong id type
    // at any call site is a compile error. The compile-fail trybuild
    // fixtures cover the rejection contract; these call sites cover
    // the positive side — they only type-check because the IDs being
    // passed around match the parameter types.
    use common::snowflake_chat::operations;

    let sent = operations::send_message(
        db,
        general.id,
        alice_id,
        "hello via operations".to_string(),
    )
    .await?;
    assert_eq!(sent.author_id, alice_id);

    let author = operations::find_message_author(db, sent.id)
        .await?
        .expect("inserted message must have an author");
    assert_eq!(author, alice_id);

    let listed_replies = operations::list_replies_to(db, first.id).await?;
    assert_eq!(listed_replies.len(), 1);

    // Composite delete via typed components in the correct positional order.
    let ban_res = operations::ban_user_from_guild(db, guild_id, alice_id).await?;
    assert_eq!(ban_res.rows_affected, 1);

    Ok(())
}
