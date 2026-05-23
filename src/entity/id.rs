//! Phantom-typed primary-key handle.
//!
//! `Id<E, T>` wraps a primary-key value of underlying type `T` and tags it
//! with the entity `E` at the type level. Two `Id` types with different
//! entity tags are never inter-convertible, so the compiler rejects
//! cross-entity ID confusion at use sites — e.g. passing a
//! `Id<post::Entity, _>` to `user::Entity::find_by_id` is a type error.
//!
//! ## Why two type parameters?
//!
//! A single-parameter `Id<E>` design (where the inner type lives behind an
//! associated trait) ran into a recursive-type problem: when a model field
//! is `pub id: CakeId` and `CakeId = Id<Entity>`, the entity's
//! `PrimaryKey::ValueType` is `Id<Entity>`, and `Id<Entity>::value: Inner`
//! would have to be `Id<Entity>` — an infinite type. Spelling out `T` in
//! the alias (`pub type CakeId = sea_orm::Id<Entity, i32>;`) sidesteps it
//! cleanly: `T` is always the raw scalar.
//!
//! ## Usage
//!
//! ```ignore
//! use sea_orm::entity::prelude::*;
//!
//! // Codegen emits this as a one-line alias per entity:
//! pub type CakeId = sea_orm::Id<Entity, i32>;
//!
//! // The model field uses the alias:
//! pub struct Model {
//!     pub id: CakeId,
//!     pub name: String,
//! }
//!
//! // Construction is explicit — `Id::new` (no `From<i32>` blanket):
//! let id = CakeId::new(7);
//!
//! // Queries use the typed handle:
//! let cake = cake::Entity::find_by_id(id).one(db).await?;
//! ```
//!
//! ## Safety contract
//!
//! `Id<E, T>` deliberately does NOT impl `From<T>` for any specific scalar.
//! The only construction path is [`Id::new`]. This is what makes
//! `user::Entity::find_by_id(7_i32)` fail to compile when the entity's PK
//! is `Id<user::Entity, i32>`: there's no `i32: Into<Id<user::Entity, i32>>`
//! impl.
//!
//! Note however, users can still explicitly write:
//!
//! ```ignore
//! impl From<i32> for sea_orm::Id<crate::cake::Entity, i32> {
//!     fn from(n: i32) -> Self { sea_orm::Id::new(n) }
//! }
//! ```
//!
//! ...and re-enable `cake::Entity::find_by_id(7_i32)` via the `Into` chain.

use crate::{
    ColIdx, DbErr, EntityTrait, PrimaryKeyTrait, QueryResult, TryFromU64, TryGetError, TryGetable,
};
use sea_query::{ArrayType, ColumnType, Nullable, Value, ValueType, ValueTypeErr};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Phantom-typed wrapper around a primary-key value.
///
/// `E` is the entity this id belongs to (a marker — never stored at
/// runtime). `T` is the raw stored value:
/// - For unary PKs, the scalar type (`i32`, `Uuid`, `String`, …).
/// - For composite PKs, a tuple of the typed components
///   (e.g. `(super::cake::CakeId, super::filling::FillingId)`).
///
/// See the [module-level docs](self) for usage and the safety contract.
#[repr(transparent)]
pub struct Id<E: EntityTrait, T> {
    /// The raw stored value.
    pub value: T,
    // `PhantomData<fn(E) -> E>` makes `E` invariant: the function-pointer
    // type has `E` in both contravariant (parameter) and covariant (return)
    // position, which combine to invariant. This is what we want — the
    // compiler must never widen an `Id<A, _>` to an `Id<B, _>` even if A
    // and B are related. `fn() -> E` alone would be covariant; `fn(E)`
    // alone would be contravariant; the combined form is the canonical
    // way to spell invariance. Function-pointer types are unconditionally
    // `Send + Sync`, so this preserves auto-traits.
    _marker: PhantomData<fn(E) -> E>,
}

impl<E: EntityTrait, T> Id<E, T> {
    /// Wrap a raw value as a typed entity ID. This is the only construction
    /// path — there is no `From<T>` blanket impl, which is what gives
    /// `Id<E, T>` its type-safety contract.
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _marker: PhantomData,
        }
    }

    /// Unwrap to the raw stored value, consuming the wrapper.
    pub fn into_inner(self) -> T {
        self.value
    }
}

// Manual impls: deriving would (incorrectly) require `E: Clone` etc., bounds
// on the phantom rather than the stored value.

impl<E: EntityTrait, T: Clone> Clone for Id<E, T> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<E: EntityTrait, T: Copy> Copy for Id<E, T> {}

impl<E: EntityTrait, T: fmt::Debug> fmt::Debug for Id<E, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Include the entity tag so `Id<post::Entity, _>(7)` and
        // `Id<user::Entity, _>(7)` don't look identical in logs — that
        // defeats the entire reason this wrapper exists.
        //
        // Every entity struct is named `Entity` by convention, so the
        // disambiguating part is the module that contains it. We render
        // `<parent_module>::<EntityName>` — the last two `::`-segments
        // of `std::any::type_name::<E>()`. Full paths are too verbose
        // for log lines; the trailing two segments preserve the
        // disambiguation while staying readable.
        let full = std::any::type_name::<E>();
        let mut tail = full.rsplitn(3, "::");
        let last = tail.next().unwrap_or(full);
        let prev = tail.next();
        let label = match prev {
            Some(p) => format!("{p}::{last}"),
            None => last.to_owned(),
        };
        write!(f, "Id<{label}>({:?})", self.value)
    }
}

impl<E: EntityTrait, T: PartialEq> PartialEq for Id<E, T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<E: EntityTrait, T: Eq> Eq for Id<E, T> {}

impl<E: EntityTrait, T: Hash> Hash for Id<E, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<E: EntityTrait, T: fmt::Display> fmt::Display for Id<E, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

// === PrimaryKeyTrait::ValueType bounds ======================================
//
// All five trait bounds delegate to `T`.
//
// `Into<Value>` (for unary `T`) → bridges to sea-query's blanket
// `impl<V: Into<Value>> From<V> for ValueTuple`, which auto-derives
// `Id<E, T>: IntoValueTuple` (the supertrait of FromValueTuple etc.).
// Composite PKs don't use `Id<E, tuple>` at the value-binding level —
// each composite component is itself a unary `Id<parent, scalar>` and
// the tuple is just `(CakeId, FillingId)`.

impl<E: EntityTrait, T> From<Id<E, T>> for Value
where
    T: Into<Value>,
{
    fn from(id: Id<E, T>) -> Self {
        id.value.into()
    }
}

// `FromValueTuple` is provided automatically by sea-query's blanket
// `impl<V: Into<Value> + ValueType> FromValueTuple for V` once we impl
// `Into<Value>` (above) and `ValueType` (below). For composite `T` neither
// bound is met and the blanket doesn't fire — that's intentional, as we
// never use `Id<E, tuple>` at the value-binding level.

// `TryGetable` (single-column read). When `T: TryGetable`, `Id<E, T>` reads
// from a single column position. This also auto-derives `TryGetableMany`
// for `Id<E, T>` (via the blanket `impl<X: TryGetable> TryGetableMany for X`)
// and makes tuples of `Id<E, T>` impl `TryGetableMany` via the per-arity
// macro, which is what composite PKs need.
//
// Note: `Id<E, T>` does NOT impl `TryGetable` when `T` is itself a tuple —
// the trait would need column-position arithmetic the macros don't provide
// for nested wrappers. Composite PKs use tuples of unary `Id<E, scalar>`,
// not `Id<E, tuple>`, so this restriction is fine in practice.
impl<E: EntityTrait, T: TryGetable> TryGetable for Id<E, T> {
    fn try_get_by<I: ColIdx>(res: &QueryResult, idx: I) -> Result<Self, TryGetError> {
        T::try_get_by(res, idx).map(Id::new)
    }
}

impl<E: EntityTrait, T: TryFromU64> TryFromU64 for Id<E, T> {
    fn try_from_u64(n: u64) -> Result<Self, DbErr> {
        T::try_from_u64(n).map(Id::new)
    }
}

// `PrimaryKeyArity` is auto-derived via the existing blanket
// `impl<V: TryGetable> PrimaryKeyArity for V { const ARITY = 1 }`. We don't
// add a direct impl because that would conflict with the blanket.

// `sea_query::ValueType` so the `DeriveEntityModel` macro can call
// `<CakeId as ValueType>::column_type()` to determine the SQL column type.
// Only available when `T: ValueType` — i.e. T is a single scalar, not a
// composite tuple. For composite PKs the macro asks each individual column
// for its type, and each column's `T` is a single scalar.
impl<E: EntityTrait, T: ValueType> ValueType for Id<E, T> {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        T::try_from(v).map(Id::new)
    }

    fn type_name() -> String {
        T::type_name()
    }

    fn array_type() -> ArrayType {
        T::array_type()
    }

    fn column_type() -> ColumnType {
        T::column_type()
    }
}

// `Nullable` so the macro can wrap the column in `Option<Id<E, T>>` for
// nullable FK columns.
impl<E: EntityTrait, T: Nullable> Nullable for Id<E, T> {
    fn null() -> Value {
        T::null()
    }
}

// === Construction note ======================================================
//
// `Id::new(value)` is the only construction path. We deliberately do NOT
// provide `impl<E, T> From<T> for Id<E, T>` — that would re-open the safety
// hole the type is designed to prevent.

// === FindByIdArg ============================================================
//
// `find_by_id` / `filter_by_id` accept anything convertible to the entity's
// primary-key value type. We could bound that directly with `Into`, but doing
// so makes the compiler's "this argument is wrong" diagnostic incomprehensible
// — it reads something like
//   `the trait bound `Id<user::Entity, i32>: From<Id<post::Entity, i32>>`
//    is not satisfied`,
// burying the two entity types inside generic args of `Into`.
//
// `FindByIdArg<E>` is a thin sea-orm-owned wrapper around that same `Into`
// bound. It exists solely so we can attach `#[diagnostic::on_unimplemented]`
// to it — diagnostics can't be attached to `Into` (a std trait). The blanket
// impl forwards through `Into`, so every existing call site still works
// without change. When the bound *fails*, the user sees a message that names
// the entity and the argument type directly.
//
// MSRV is 1.85; `#[diagnostic::on_unimplemented]` is stable since 1.78.

/// Helper bound used by `find_by_id` / `filter_by_id`.
///
/// Implemented for every `T` that converts into `E`'s primary-key value type
/// via `Into`. This trait exists to provide a better compiler error than the
/// raw `Into` bound when the argument doesn't match — see the module docs.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a primary-key argument for `{E}`",
    label = "expected `{E}`'s `PrimaryKey::ValueType` (or something convertible to it), got `{Self}`",
    note = "type-safe `Id<E, _>` wrappers deliberately do not impl `From<inner>` to prevent cross-entity ID confusion. Construct ids explicitly with `Id::new(..)` (or the per-entity alias's `::new`), and pass an id belonging to this entity."
)]
pub trait FindByIdArg<E: EntityTrait>: Sized {
    /// Convert this argument into the entity's primary-key value tuple.
    fn into_pk_value(self) -> <E::PrimaryKey as PrimaryKeyTrait>::ValueType;
}

// `do_not_recommend` (stable 1.85) tells rustc not to surface this blanket impl
// in error messages when its where-clause fails. Without it, the user sees a
// confusing message about `From<Id<post::Entity, _>>` not being implemented
// for `Id<user::Entity, _>` — the deeper sub-bound — instead of the
// `on_unimplemented` message on `FindByIdArg` itself.
#[diagnostic::do_not_recommend]
impl<E: EntityTrait, T> FindByIdArg<E> for T
where
    T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
{
    fn into_pk_value(self) -> <E::PrimaryKey as PrimaryKeyTrait>::ValueType {
        self.into()
    }
}

// === Serde ==================================================================
//
// Transparent: `Id<E, T>` serializes as just the inner `T`, not as a
// wrapper object. Gated behind `with-json` like the rest of sea-orm's serde
// surface (see `entity/compound/has_one.rs` for the same pattern).

#[cfg(feature = "with-json")]
impl<E: EntityTrait, T: serde::Serialize> serde::Serialize for Id<E, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.serialize(serializer)
    }
}

#[cfg(feature = "with-json")]
impl<'de, E: EntityTrait, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Id<E, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Id::new)
    }
}
