//! Snowflake-chat fixture.
//!
//! Discord-shaped data model that demonstrates where per-entity PK
//! newtypes earn their keep. Every PK is an `i64` at the SQL level —
//! Discord-style snowflakes — but the entities all carry distinct
//! Rust types so cross-entity confusion is a compile error.
//!
//! Highlights that this fixture exercises (and that blogger does not):
//!
//! - **Multiple FKs to the same parent in one table.** `message` has
//!   both `author_id` and `mention_user_id` referencing `user.id`;
//!   `dm_thread` has `participant_a` and `participant_b` doing the
//!   same. Each is given a per-column role wrapper so a swap at a
//!   call site is a type error.
//! - **Self-reference.** `message.reply_to_message_id` points back at
//!   `message`.
//! - **Composite primary keys** with typed components (`member` keyed
//!   by `(GuildId, UserId)`, `reaction` keyed by
//!   `(MessageId, UserId, String)`).
//! - **No explicit `#[sea_orm(auto_increment)]` anywhere.** Every
//!   single-PK entity relies on `PkAutoIncrementHint` to resolve the
//!   default through `DeriveValueType` to the inner `i64` and back to
//!   `true`. If trait resolution regresses, schema creation here
//!   stops auto-generating IDs and the runtime tests fail.

pub mod channel;
pub mod dm_thread;
pub mod guild;
pub mod member;
pub mod message;
pub mod reaction;
pub mod user;
