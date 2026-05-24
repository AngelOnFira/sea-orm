use sea_orm::entity::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct DmThreadId(pub i64);

// Role wrappers: both columns FK to `user.id`. Distinct types make
// `DmThread { participant_a: bob, participant_b: alice, .. }` distinguishable
// from the swap at type-check time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct DmThreadParticipantA(pub super::user::UserId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
pub struct DmThreadParticipantB(pub super::user::UserId);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "snowflake_dm_thread")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: DmThreadId,
    pub participant_a: DmThreadParticipantA,
    pub participant_b: DmThreadParticipantB,
}

impl ActiveModelBehavior for ActiveModel {}
