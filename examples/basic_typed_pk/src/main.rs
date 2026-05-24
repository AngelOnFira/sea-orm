//! Typed-PK chat example.
//!
//! Everything under `src/entity/` was produced by
//! `sea-orm-cli generate entity --with-pk-newtypes` against
//! `chat.sql`. The shape of those files — `pub type FooId =
//! sea_orm::Id<Entity, i64>;` aliases, role wrappers on the
//! `user_follower` junction, FK columns typed as parent aliases — is
//! exactly what codegen emits today. This example exercises that
//! generated code end-to-end against in-memory SQLite.
//!
//! Regenerate with:
//!
//!     sqlite3 /tmp/typed_pk_chat.db < examples/basic_typed_pk/chat.sql
//!     sea-orm-cli generate entity \
//!         --database-url sqlite:///tmp/typed_pk_chat.db \
//!         --with-pk-newtypes \
//!         --output-dir examples/basic_typed_pk/src/entity

mod entity;
mod operations;

use entity::{channel, guild, message, user, user_follower};
use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ConnectOptions, ConnectionTrait, Database, DbBackend, DbErr,
    Schema, Statement,
};

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    let db = Database::connect(ConnectOptions::new("sqlite::memory:")).await?;
    create_schema(&db).await?;

    // Insert two users and one guild. PKs are typed `UserId` / `GuildId`
    // straight out of `.insert(...)` — no raw `i64` ever appears here.
    let alice = user::ActiveModel {
        username: Set("alice".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let bob = user::ActiveModel {
        username: Set("bob".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let guild_a = guild::ActiveModel {
        name: Set("Cooks United".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;
    let general = channel::ActiveModel {
        guild_id: Set(guild_a.id),
        name: Set("general".to_string()),
        ..Default::default()
    }
    .insert(&db)
    .await?;

    // Send two messages via the typed domain layer. `send_message` only
    // accepts `ChannelId` + `UserId`; passing a `GuildId` is a compile error.
    let first = operations::send_message(&db, general.id, alice.id, "hello world".to_string()).await?;
    let reply = operations::send_message_with_mention(
        &db,
        general.id,
        bob.id,
        alice.id,
        Some(first.id),
        "hi alice".to_string(),
    )
    .await?;

    println!("first message: {first:?}");
    println!("reply: {reply:?}");

    // Composite-PK lookup: each component is its own typed alias.
    operations::add_member(&db, guild_a.id, alice.id, Some("Chef Alice".to_string())).await?;
    operations::add_member(&db, guild_a.id, bob.id, None).await?;
    let alice_member = operations::find_member(&db, guild_a.id, alice.id)
        .await?
        .expect("alice should be a member");
    println!("alice's member row: {alice_member:?}");

    // Role-wrapped junction insert. The two PK columns are distinct types
    // (`UserFollowerPkUserId`, `UserFollowerPkFollowerId`), so swapping
    // arguments at this call site would be a compile error.
    user_follower::ActiveModel {
        user_id: Set(user_follower::UserFollowerPkUserId(alice.id)),
        follower_id: Set(user_follower::UserFollowerPkFollowerId(bob.id)),
    }
    .insert(&db)
    .await?;

    // Reply chain via self-ref.
    let replies = operations::list_replies_to(&db, first.id).await?;
    println!("{} reply/replies to first message:", replies.len());
    for r in &replies {
        println!("  {r:?}");
    }

    // Composite-PK delete (ban a user from a guild).
    let res = operations::ban_user_from_guild(&db, guild_a.id, alice.id).await?;
    println!("banned alice from guild: {} row(s) affected", res.rows_affected);

    Ok(())
}

/// Create the schema by running `CREATE TABLE` statements derived from
/// the entity definitions. For a real app you'd run migrations; for an
/// in-memory example this is the lightest path.
async fn create_schema(db: &impl ConnectionTrait) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let schema = Schema::new(backend);

    // Order matters: parents before children so FKs resolve.
    for stmt in [
        schema.create_table_from_entity(guild::Entity),
        schema.create_table_from_entity(user::Entity),
        schema.create_table_from_entity(channel::Entity),
        schema.create_table_from_entity(entity::member::Entity),
        schema.create_table_from_entity(message::Entity),
        schema.create_table_from_entity(entity::reaction::Entity),
        schema.create_table_from_entity(user_follower::Entity),
    ] {
        db.execute(&stmt).await?;
    }
    // Enable FK enforcement on SQLite so the example actually exercises
    // the constraints.
    db.execute_raw(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = ON".to_string(),
    ))
    .await?;
    Ok(())
}
