//! Positive tests for `PkAutoIncrementHint` resolution.
//!
//! These pin the contract that `DeriveEntityModel` emits the trait call
//! correctly and that the trait propagates through `DeriveValueType`
//! wrappers and `Id<E, T>` aliases.

use sea_orm::{DeriveValueType, Id, PkAutoIncrementHint, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct IntegerWrapper(pub i64);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct StringWrapper(pub String);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct NestedIntegerWrapper(pub IntegerWrapper);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct NestedStringWrapper(pub StringWrapper);

mod ent_for_id {
    use super::*;
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "ent_for_id")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}
    impl ActiveModelBehavior for ActiveModel {}
}

#[test]
fn primitive_integer_defaults_true() {
    assert!(<i32 as PkAutoIncrementHint>::IS_AUTO);
    assert!(<i64 as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn primitive_string_defaults_false() {
    assert!(!<String as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_integer() {
    assert!(<IntegerWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_string() {
    assert!(!<StringWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn value_type_wrapper_propagates_through_nested() {
    assert!(<NestedIntegerWrapper as PkAutoIncrementHint>::IS_AUTO);
    assert!(!<NestedStringWrapper as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn id_alias_propagates_through_inner() {
    type IntId = Id<ent_for_id::Entity, i32>;
    type StrId = Id<ent_for_id::Entity, String>;
    assert!(<IntId as PkAutoIncrementHint>::IS_AUTO);
    assert!(!<StrId as PkAutoIncrementHint>::IS_AUTO);
}

#[test]
fn entity_with_i32_pk_resolves_true() {
    assert!(<ent_for_id::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}

#[cfg(feature = "with-uuid")]
#[test]
fn uuid_defaults_false() {
    assert!(!<uuid::Uuid as PkAutoIncrementHint>::IS_AUTO);
}
