use sea_orm::entity::prelude::*;

pub type DmThreadId = sea_orm::Id<Entity, i64>;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_dm_thread")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: DmThreadId,
    pub participant_a: super::user::UserId,
    pub participant_b: super::user::UserId,
}

impl ActiveModelBehavior for ActiveModel {}
