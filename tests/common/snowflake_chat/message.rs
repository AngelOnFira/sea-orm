use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct MessageId(pub i64);

// Role wrappers for the two FK columns that both target `user.id`. Wrapping
// each in a distinct struct makes a swap at a call site
// (`message::Model { author_id: mention, mention_user_id: author, ... }`)
// a type error rather than a silent data bug.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct MessageAuthorId(pub super::user::UserId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct MessageMentionUserId(pub super::user::UserId);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: MessageId,
    pub channel_id: super::channel::ChannelId,
    pub author_id: MessageAuthorId,
    pub mention_user_id: Option<MessageMentionUserId>,
    pub reply_to_message_id: Option<MessageId>,
    pub content: String,
    #[sea_orm(belongs_to, from = "channel_id", to = "id")]
    pub channel: HasOne<super::channel::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
