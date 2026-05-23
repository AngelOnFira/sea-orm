use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, DeriveValueType)]
pub struct GuildId(pub i64);

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
