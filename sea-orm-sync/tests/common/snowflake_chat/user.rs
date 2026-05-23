use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, DeriveValueType)]
pub struct UserId(pub i64);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: UserId,
    #[sea_orm(unique)]
    pub username: String,
}

impl ActiveModelBehavior for ActiveModel {}
