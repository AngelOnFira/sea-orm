use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub post_id: super::post::PostId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: super::tag::TagId,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: Option<super::post::Entity>,
    #[sea_orm(belongs_to, from = "tag_id", to = "id")]
    pub tag: Option<super::tag::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
