use sea_orm::entity::prelude::*;

pub type CommentId = sea_orm::Id<Entity, i32>;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
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
