//! Passing a raw `i32` to `find_by_id` must not compile when the PK is a
//! newtype with no `From<i32>` impl. This is the core safety contract.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct UserId(pub i32);

mod user {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: UserId,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() {
    let _ = user::Entity::find_by_id(1i32);
}
