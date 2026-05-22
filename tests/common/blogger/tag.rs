use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct TagId(pub i32);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment)]
    pub id: TagId,
    pub tag: String,
    #[sea_orm(has_many, via = "post_tag")]
    pub posts: HasMany<super::post::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
