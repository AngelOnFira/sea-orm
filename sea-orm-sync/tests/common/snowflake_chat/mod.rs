//! Snowflake-chat fixture.
//!
//! Discord-shaped data model where every PK is an `i64` at the SQL level
//! — Discord-style snowflakes — wrapped in a distinct `DeriveValueType`
//! newtype per entity. Cross-entity ID confusion is a compile error.
//!
//! This fixture exercises four patterns:
//!
//! - **Multiple FKs to the same parent in one table.** `message` has
//!   both `author_id` and `mention_user_id` referencing `user.id`;
//!   `dm_thread` has `participant_a` and `participant_b` doing the
//!   same. Each column carries a per-column role wrapper so a swap at
//!   a call site is a type error.
//! - **Self-reference.** `message.reply_to_message_id` points back at
//!   `message`.
//! - **Composite primary keys** with typed components (`member` keyed
//!   by `(GuildId, UserId)`, `reaction` keyed by
//!   `(MessageId, UserId, String)`).
//! - **Trait-resolved auto-increment.** No single-PK entity carries an
//!   explicit `#[sea_orm(auto_increment)]` annotation. Each PK column
//!   resolves the default through `PkAutoIncrementHint` →
//!   `DeriveValueType` wrapper → inner `i64` → `true`, so schema
//!   creation emits `AUTOINCREMENT` and `insert(...)` calls can omit
//!   `id`.

pub mod channel;
pub mod dm_thread;
pub mod guild;
pub mod member;
pub mod message;
pub mod reaction;
pub mod user;
