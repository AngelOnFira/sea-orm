use sea_orm::entity::prelude::*;

pub type AttachmentId = sea_orm::Id<Entity, i32>;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "attachment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: AttachmentId,
    pub post_id: Option<super::post::PostId>,
    pub file: String,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: HasOne<super::post::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
