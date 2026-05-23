use sea_orm::entity::prelude::*;

// Role wrappers: both PK columns FK to `user.id`, so they share the parent
// type `super::user::UserId`. Wrapping each in a distinct struct makes the
// columns type-distinct at the call site, so a swap like
// `find_by_id((follower_role, user_role))` fails to compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
#[sea_orm(try_from_u64)]
pub struct UserFollowerPkUserId(pub super::user::UserId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, DeriveValueType)]
#[sea_orm(try_from_u64)]
pub struct UserFollowerPkFollowerId(pub super::user::UserId);

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_follower")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: UserFollowerPkUserId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub follower_id: UserFollowerPkFollowerId,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: Option<super::user::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "Follower",
        from = "follower_id",
        to = "id"
    )]
    pub follower: Option<super::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
