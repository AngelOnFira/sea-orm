//! Per-type hint for whether a primary key column defaults to
//! `AUTO_INCREMENT` when the entity declaration doesn't specify
//! explicitly.
//!
//! The `DeriveEntityModel` macro consults this trait at trait-resolution
//! time. Wrapper types and `Id<E, T>` aliases resolve through their inner
//! type because each emits a delegating impl of [`DelegatesPkAutoIncrementHint`].
//! For example:
//!
//! - `pub id: i64` resolves to `true` via the integer-primitive impl.
//! - `pub id: RoleId` where `RoleId(pub i64)` is `DeriveValueType`
//!   resolves to `true` by delegating through the inner `i64`.
//! - `pub id: Token` where `Token(pub String)` resolves to `false`
//!   through `String`.
//! - `pub id: CakeId` where `CakeId = Id<Entity, i32>` resolves to
//!   `true` via the `Id<E, T>` blanket and the inner `i32` impl.
//!
//! For custom PK types that don't fall into any of these categories,
//! either impl this trait on the type or specify `auto_increment`
//! explicitly on the column attribute.

use crate::EntityTrait;

/// Default `auto_increment` value used by the entity macro when a
/// primary key column does not specify `#[sea_orm(auto_increment = ...)]`
/// explicitly.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a primary key without an explicit `auto_increment` setting",
    note = "the entity macro looks up the default via `sea_orm::PkAutoIncrementHint`",
    note = "either impl `sea_orm::PkAutoIncrementHint` for `{Self}`, or specify \
            `#[sea_orm(primary_key, auto_increment = false)]` (or `= true`) on the column"
)]
pub trait PkAutoIncrementHint {
    /// Whether columns of this type default to `AUTO_INCREMENT` when used
    /// as a primary key.
    const IS_AUTO: bool;
}

macro_rules! impl_auto_true {
    ($($t:ty),* $(,)?) => {
        $(
            impl PkAutoIncrementHint for $t {
                const IS_AUTO: bool = true;
            }
        )*
    };
}

macro_rules! impl_auto_false {
    ($($t:ty),* $(,)?) => {
        $(
            impl PkAutoIncrementHint for $t {
                const IS_AUTO: bool = false;
            }
        )*
    };
}

impl_auto_true!(i8, i16, i32, i64, u8, u16, u32, u64, isize, usize);
impl_auto_false!(String, Vec<u8>);

#[cfg(feature = "with-uuid")]
impl PkAutoIncrementHint for uuid::Uuid {
    const IS_AUTO: bool = false;
}

#[cfg(feature = "with-uuid")]
mod uuid_fmt_impls {
    use super::PkAutoIncrementHint;
    use uuid::fmt;

    impl PkAutoIncrementHint for fmt::Braced {
        const IS_AUTO: bool = false;
    }
    impl PkAutoIncrementHint for fmt::Hyphenated {
        const IS_AUTO: bool = false;
    }
    impl PkAutoIncrementHint for fmt::Simple {
        const IS_AUTO: bool = false;
    }
    impl PkAutoIncrementHint for fmt::Urn {
        const IS_AUTO: bool = false;
    }
}

/// Internal helper trait: marks a wrapper as delegating its
/// `PkAutoIncrementHint` resolution to an inner type.
///
/// `DeriveValueType` emits an impl of this for every wrapper it
/// generates, naming the wrapped inner type. The blanket
/// `PkAutoIncrementHint` impl below then bridges from this trait,
/// keeping the inner-type bound on a generic parameter (deferred at
/// trait-resolution time) instead of a concrete-type `where` clause
/// (eagerly checked at impl-declaration time, which would force a
/// compile error on every wrapper whose inner doesn't impl
/// `PkAutoIncrementHint`, even if the wrapper is never used as a PK).
pub trait DelegatesPkAutoIncrementHint {
    /// The inner type whose `PkAutoIncrementHint` impl is delegated to.
    type Inner: ?Sized;
}

impl<T> PkAutoIncrementHint for T
where
    T: DelegatesPkAutoIncrementHint,
    T::Inner: PkAutoIncrementHint,
{
    const IS_AUTO: bool = <T::Inner as PkAutoIncrementHint>::IS_AUTO;
}

/// `Id<E, T>` delegates to its inner `T`.
impl<E, T> DelegatesPkAutoIncrementHint for crate::Id<E, T>
where
    E: EntityTrait,
{
    type Inner = T;
}

#[cfg(test)]
mod tests {
    use super::PkAutoIncrementHint;

    #[test]
    fn integer_primitives_default_true() {
        assert!(<i32 as PkAutoIncrementHint>::IS_AUTO);
        assert!(<i64 as PkAutoIncrementHint>::IS_AUTO);
        assert!(<u32 as PkAutoIncrementHint>::IS_AUTO);
        assert!(<usize as PkAutoIncrementHint>::IS_AUTO);
    }

    #[test]
    fn string_and_bytes_default_false() {
        assert!(!<String as PkAutoIncrementHint>::IS_AUTO);
        assert!(!<Vec<u8> as PkAutoIncrementHint>::IS_AUTO);
    }

    #[cfg(feature = "with-uuid")]
    #[test]
    fn uuid_defaults_false() {
        assert!(!<uuid::Uuid as PkAutoIncrementHint>::IS_AUTO);
    }
}
