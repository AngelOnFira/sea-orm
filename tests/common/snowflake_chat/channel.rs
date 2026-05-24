use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct ChannelId(pub i64);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_channel")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: ChannelId,
    pub guild_id: super::guild::GuildId,
    pub name: String,
    #[sea_orm(belongs_to, from = "guild_id", to = "id")]
    pub guild: HasOne<super::guild::Entity>,
    #[sea_orm(has_many)]
    pub messages: HasMany<super::message::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
