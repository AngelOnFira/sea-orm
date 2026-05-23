use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_reaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub message_id: super::message::MessageId,
    #[sea_orm(primary_key)]
    pub user_id: super::user::UserId,
    #[sea_orm(primary_key)]
    pub emoji: String,
    #[sea_orm(belongs_to, from = "message_id", to = "id")]
    pub message: Option<super::message::Entity>,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: Option<super::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
