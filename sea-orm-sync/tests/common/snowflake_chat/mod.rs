//! Snowflake-chat fixture.
//!
//! Discord-shaped data model where every entity's PK is a per-entity
//! `sea_orm::Id<Entity, i64>` alias — the exact shape that
//! `sea-orm-cli generate --with-pk-newtypes` produces. The
//! `pk_newtypes_snowflake_chat_shape` codegen test in
//! `sea-orm-codegen/src/entity/writer.rs` pins that contract from the
//! other direction: it runs codegen against a matching schema and
//! asserts the output has the same shape this fixture is hand-written
//! to match. If codegen drifts, that test fails; if this fixture is
//! ever rewritten in a way codegen wouldn't produce, the runtime tests
//! here no longer prove generated code works.
//!
//! Patterns exercised:
//!
//! - **Multiple FKs to the same parent in one table.** `message` has
//!   both `author_id` and `mention_user_id` referencing `user.id`;
//!   `dm_thread` has `participant_a` and `participant_b` doing the
//!   same. Both columns share the parent's `UserId` type (codegen
//!   doesn't wrap non-PK FK columns).
//! - **Self-reference.** `message.reply_to_message_id` points back at
//!   `message` and resolves to the local `Option<MessageId>`.
//! - **Composite primary keys** with typed components (`member` keyed
//!   by `(GuildId, UserId)`, `reaction` keyed by
//!   `(MessageId, UserId, String)`).
//! - **Trait-resolved auto-increment.** No single-PK entity carries an
//!   explicit `#[sea_orm(auto_increment)]` annotation. Each PK column
//!   resolves the default through `PkAutoIncrementHint` →
//!   `Id<E, T>` → inner `i64` → `true`, so schema creation emits
//!   `AUTOINCREMENT` and `insert(...)` calls can omit `id`.
//!
//! See [`operations`] for realistic typed-PK domain code exercised by
//! the integration test.

pub mod channel;
pub mod dm_thread;
pub mod guild;
pub mod member;
pub mod message;
pub mod operations;
pub mod reaction;
pub mod user;
