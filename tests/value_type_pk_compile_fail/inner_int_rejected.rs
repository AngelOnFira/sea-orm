//! Validates the `find_by_id` tightening: passing a different integer
//! width than the PK's `ValueType` no longer compiles. Previously
//! `Into<i32>` allowed `1u8` etc. through.

use sea_orm::entity::prelude::*;

mod cake {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "cake")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() {
    let _ = cake::Entity::find_by_id(1u8);
}
