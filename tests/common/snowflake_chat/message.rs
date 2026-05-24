use sea_orm::entity::prelude::*;

pub type MessageId = sea_orm::Id<Entity, i64>;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: MessageId,
    pub channel_id: super::channel::ChannelId,
    pub author_id: super::user::UserId,
    pub mention_user_id: Option<super::user::UserId>,
    pub reply_to_message_id: Option<MessageId>,
    pub content: String,
    #[sea_orm(belongs_to, from = "channel_id", to = "id")]
    pub channel: HasOne<super::channel::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
