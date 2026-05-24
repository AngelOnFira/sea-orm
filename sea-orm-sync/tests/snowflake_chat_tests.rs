//! Minimal pin for `PrimaryKeyTrait::auto_increment()` resolution
//! through `PkAutoIncrementHint` for entities whose PK is a wrapper
//! type (`DeriveValueType` newtype or `sea_orm::Id<E, T>` alias).
//!
//! End-to-end coverage of typed-PK codegen output through the entire
//! ORM stack (CRUD, composite-PK lookup, self-ref query, role
//! wrappers, domain-code threading) lives in
//! `examples/basic_typed_pk/`. That example uses actual codegen
//! output, not a hand-written fixture.
//!
//! This test is kept narrow on purpose: a no-DB unit-style assertion
//! catches a regression in the macro-level trait wiring before any
//! integration suite has a chance to run.

pub mod common;

use sea_orm::{Id, PkAutoIncrementHint, PrimaryKeyTrait, entity::prelude::*};

mod fixture {
    use sea_orm::entity::prelude::*;

    // Per-entity `Id<E, T>` alias (the shape `sea-orm-cli generate
    // entity --with-pk-newtypes` produces for single-PK tables).
    pub type GuildId = sea_orm::Id<Entity, i64>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "snowflake_guild")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: GuildId,
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod fixture_composite {
    use sea_orm::entity::prelude::*;

    // Composite PK: each component is itself a typed alias from another
    // entity. The macro must short-circuit to `false` regardless of how
    // the trait would resolve for the individual columns.
    pub type ParentAId = sea_orm::Id<Entity, i64>;
    pub type ParentBId = sea_orm::Id<Entity, i64>;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "snowflake_member")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub guild_id: ParentAId,
        #[sea_orm(primary_key)]
        pub user_id: ParentBId,
        pub nickname: Option<String>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

/// Pin: `Id<E, i64>` resolves through `PkAutoIncrementHint` to `true`.
/// If the macro stops emitting the trait call (or the blanket impl on
/// `Id<E, T>` regresses), this fails before any DB is involved.
#[test]
fn id_alias_pk_defaults_to_auto_increment() {
    assert!(<fixture::PrimaryKey as PrimaryKeyTrait>::auto_increment());
    assert!(<Id<fixture::Entity, i64> as PkAutoIncrementHint>::IS_AUTO);
    assert!(<Id<fixture::Entity, i32> as PkAutoIncrementHint>::IS_AUTO);
    assert!(!<Id<fixture::Entity, String> as PkAutoIncrementHint>::IS_AUTO);
}

/// Pin: composite PKs always emit `auto_increment() == false`,
/// regardless of how the trait would resolve for the individual
/// component types.
#[test]
fn composite_pk_is_never_auto_increment() {
    assert!(!<fixture_composite::PrimaryKey as PrimaryKeyTrait>::auto_increment());
}
