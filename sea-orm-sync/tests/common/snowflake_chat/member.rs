use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_member")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub guild_id: super::guild::GuildId,
    #[sea_orm(primary_key)]
    pub user_id: super::user::UserId,
    pub nickname: Option<String>,
    #[sea_orm(belongs_to, from = "guild_id", to = "id")]
    pub guild: Option<super::guild::Entity>,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: Option<super::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
