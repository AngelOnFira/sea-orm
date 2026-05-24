//! Shape mirrors what `sea-orm-cli generate --with-pk-newtypes` produces
//! for a table named `snowflake_guild` with an `i64` PK.
//! See the `pk_newtypes_snowflake_chat_shape` codegen test in
//! `sea-orm-codegen/src/entity/writer.rs` for the contract.

use sea_orm::entity::prelude::*;

pub type GuildId = sea_orm::Id<Entity, i64>;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_guild")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: GuildId,
    pub name: String,
    #[sea_orm(has_many)]
    pub channels: HasMany<super::channel::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
