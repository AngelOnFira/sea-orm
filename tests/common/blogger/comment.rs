use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct CommentId(pub i32);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: CommentId,
    pub comment: String,
    pub user_id: super::user::UserId,
    pub post_id: super::post::PostId,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::user::Entity>,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: HasOne<super::post::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
