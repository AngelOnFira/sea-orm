# Type-safe primary keys via per-entity newtypes

Adds an opt-in code-generation mode (`sea-orm-cli generate entity
--with-pk-newtypes`) and a small library surface (`sea_orm::Id<E, T>`,
`PkAutoIncrementHint`, `FindByIdArg`) that together give compile-time
protection against mixing up primary-key values across entities.

Backwards-compatible: nothing changes for entities generated without
the flag.

---

## What it solves

Today's API:

```rust
mod user { pub struct Model { pub id: i32, .. } }
mod post { pub struct Model { pub id: i32, .. } }

let post: post::Model = /* ... */;
let owner = user::Entity::find_by_id(post.id).one(db).await?;
```

This compiles. It silently looks up the user whose id happens to
equal a post id. Same shape appears in domain code
(`fn ban_user(user_id: i32)` accepting a `post.id`), in junction
inserts (`user_follower::ActiveModel { user_id, follower_id }` where
the two args are swapped), and in any FK column that carries a raw
scalar (`comment::Model { post_id: i32, user_id: i32 }`).

With this PR's opt-in:

```rust
let owner = user::Entity::find_by_id(post.id).one(db).await?;
//                                   ^^^^^^^
//                                   error: `PostId` cannot be used as
//                                   a primary-key argument for `user::Entity`
```

The compile error fires because `post.id` is now `PostId`
(`Id<post::Entity, i32>`), which has no `Into<Id<user::Entity, i32>>`
impl by design.

---

## The pieces

### 1. `sea_orm::Id<E, T>` — phantom-typed PK wrapper

Located at `src/entity/id.rs`. Roughly:

```rust
#[repr(transparent)]
pub struct Id<E: EntityTrait, T> {
    pub value: T,
    _marker: PhantomData<fn(E) -> E>,   // invariant in E
}

impl<E, T> Id<E, T> { pub const fn new(v: T) -> Self { .. } }
```

The safety contract lives on one deliberate omission: **there is no
`impl<E, T> From<T> for Id<E, T>` blanket**. The only construction
path is `Id::new(value)`. That single design choice is what makes
the compiler reject cross-entity confusion at every use site.

Blanket impls on `Id<E, T>` cover everything a PK needs:
`Clone`/`Copy` (conditional on `T`), `Debug`/`PartialEq`/`Eq`/`Hash`/`Display`,
`Serialize`/`Deserialize`, `IntoValueTuple`/`FromValueTuple`,
`TryFromU64`/`TryGetable`, `ValueType`/`Nullable`,
`PkAutoIncrementHint` (delegates to `T`).

### 2. Codegen flag: `--with-pk-newtypes`

Adds a CLI flag and a `PkNewtypeFormat::Inline` writer mode. When
enabled, every entity's generated file gets an additional alias
line and each column's emitted Rust type follows a fixed precedence
rule.

### 3. `FindByIdArg<E>` trait

Located at the bottom of `src/entity/id.rs`. Used as the bound on
`find_by_id` / `filter_by_id` / `delete_by_id`:

```rust
fn find_by_id<T>(values: T) -> Select<Self>
where T: FindByIdArg<Self>,
```

`FindByIdArg<E>` is a thin sea-orm-owned wrapper around
`Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>` so we can attach
`#[diagnostic::on_unimplemented]` with a curated error message
(`#[diagnostic]` can't be attached to a std trait). The blanket impl
forwards through `Into`, so backwards behavior is preserved for
untyped entities (`find_by_id(7u8)` against an `i32` PK still works
via `u8: Into<i32>`).

### 4. `PkAutoIncrementHint` trait

Located at `src/entity/auto_increment_hint.rs`. Macro-driven
resolution of whether a PK column defaults to `AUTO_INCREMENT`:

```rust
pub trait PkAutoIncrementHint { const IS_AUTO: bool; }
impl PkAutoIncrementHint for i32 { const IS_AUTO: bool = true; }
impl PkAutoIncrementHint for String { const IS_AUTO: bool = false; }
// .. integer primitives true; String/Vec<u8>/Uuid false
```

`DeriveValueType` automatically emits a delegating impl
(`DelegatesPkAutoIncrementHint`) so wrappers like `Token(pub String)`
resolve through their inner type to the right default. Replaces a
brittle textual suffix heuristic that couldn't see through wrappers.

---

## Codegen rules (exhaustive)

For every column in a `--with-pk-newtypes` codegen pass, the emitted
Rust type is resolved in this order. Each step is checked against
the column currently being emitted (`(current_table, column_name)`).

### Step 1. Role wrapper

Triggers when the column is a **PK column** in a table where **more
than one column** FK-references the same parent table. Each such
column gets a per-column wrapper struct named
`<OwnTableCamel>Pk<ColumnCamel>`. The wrapper wraps the parent's PK
alias and carries `#[derive(DeriveValueType)]`.

Example — `user_follower` table with `user_id, follower_id` both
referencing `user.id`:

```rust
// generated user_follower.rs:
pub struct UserFollowerPkUserId(pub super::user::UserPk);
pub struct UserFollowerPkFollowerId(pub super::user::UserPk);

pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: UserFollowerPkUserId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub follower_id: UserFollowerPkFollowerId,
}
```

A positional swap at any insert site is a compile error because
the two wrappers are type-distinct.

Restricted to PK columns. Non-PK FK columns that share a parent
fall to step 2 and share the parent's alias.

### Step 2. Single-parent FK column

Triggers when the column has **exactly one** FK back-reference
recorded in `Column::refs`. Emits the parent's PK alias:

```rust
// child entity:
pub post_id: super::post::PostPk,
```

Self-referencing FK emits the local alias (no `super::`):

```rust
// message entity, reply_to_message_id -> message.id:
pub reply_to_message_id: Option<MessagePk>,
```

If the parent doesn't have a PK alias in the lookup index (rare,
e.g. composite-PK parent), falls through to step 4.

### Step 3. Own-PK alias

Triggers when the column has **no FK back-references** AND is one
of this entity's primary-key columns. Emits the local alias:

```rust
pub id: CakePk,                  // unary PK
pub project_id: ProjectMemberProjectId,  // composite PK component (non-FK case)
```

The alias is declared at the top of the entity file as
`pub type CakePk = sea_orm::Id<Entity, i32>;`.

### Step 4. Raw scalar

The fallback. Emits the inferred Rust scalar (`i32`, `String`, ...)
as if `--with-pk-newtypes` weren't enabled. Reached when:

- the column has **more than one** FK back-reference (multi-parent
  FK — see "deliberate fallback" below)
- the column has no FK back-references AND isn't a PK column
- the parent referenced by a single-FK column has no PK alias

---

## Naming rules

### PK aliases (Step 3 emission)

| PK shape | Alias name | Example |
|---|---|---|
| Unary PK | `<TableCamel>Pk` | `task` → `TaskPk` |
| Composite PK column | `<TableCamel><ColumnCamel>` | `widget(id, secondary_id)` → `WidgetId`, `WidgetSecondaryId` |

The composite rule applies a one-step collapse if the combined name
would end in `IdId` (e.g. table `cake_id` with column `id` produces
`CakeId`, not `CakeIdId`).

The previous unary convention (`<TableCamel>Id`, with `Pk` fallback
only when the table ended in `Id`) was replaced with the consistent
`<TableCamel>Pk` form in this PR.

### Role wrappers (Step 1 emission)

Always `<OwnTableCamel>Pk<ColumnCamel>`. The `Own` prefix means
multiple junction tables referencing the same parent get distinct
wrappers (e.g. `UserFollowerPkUserId` and `ChatParticipantPkUserId`
both wrap `super::user::UserPk` but are type-distinct).

Restricted to PK columns of the junction; non-PK role
disambiguation could be added later but is out of scope here.

---

## Deliberate fallback to scalar: multi-parent FK columns

When a single SQL column is FK-constrained against more than one
parent (legal SQL — appears in migration shims and audit
patterns), codegen opts out of newtyping for that column:

```sql
CREATE TABLE child (
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (user_id) REFERENCES legacy_users (id)
);
```

generates

```rust
pub struct Model {
    pub user_id: i32,   // raw scalar, not a typed alias
}
```

Rationale: no single typed alias can faithfully represent "this
column's value is an id of either parent." Picking one parent
silently would lie about the schema. A sum type
(`enum UserId { Current(UserPk), Legacy(LegacyUserPk) }`) would
force pattern-matching at every read site for what is usually a
transient migration pattern.

The `Relation` enum still emits both BelongsTo arms — only the
column field type degrades to scalar. Could be revisited as a
`Either<A, B>`-style API later if a safe path appears.

Pinned by `pk_newtypes_multi_parent_fk_falls_back_to_scalar` in
`sea-orm-codegen/src/entity/writer.rs`.

---

## `auto_increment` resolution

For each PK column, the macro emits one of three bodies for
`PrimaryKeyTrait::auto_increment()`:

1. Composite PK → literal `false`.
2. Explicit `#[sea_orm(auto_increment = ...)]` annotation → literal bool.
3. Otherwise → `<FieldType as PkAutoIncrementHint>::IS_AUTO`.

This is a trait-resolved default, not a textual suffix check. So:

- `pub id: i64` → `true` via `impl PkAutoIncrementHint for i64`.
- `pub id: String` → `false` via `impl PkAutoIncrementHint for String`.
- `pub id: RoleId` where `RoleId(pub i64)` is `DeriveValueType` → `true`
  via `DelegatesPkAutoIncrementHint` → inner `i64`.
- `pub id: Token` where `Token(pub String)` → `false`, same path
  through `String`.
- `pub id: CakePk` where `CakePk = Id<Entity, i32>` → `true` via
  the blanket impl on `Id<E, T>` → inner `i32`.

For types that don't impl `PkAutoIncrementHint` and aren't covered
by `DeriveValueType`'s auto-derived impl, the macro emits a compile
error with `#[diagnostic::on_unimplemented]` pointing the user at
either an explicit `auto_increment = ...` annotation or a manual
trait impl.

---

## `find_by_id` / `filter_by_id` / `delete_by_id`

All three signatures use the same `T: FindByIdArg<Self>` bound. The
blanket impl is `impl<E, T> FindByIdArg<E> for T where T: Into<E::
PrimaryKey::ValueType>`, so:

| Entity PK | Call | Result |
|---|---|---|
| raw `i32` | `find_by_id(7_i32)` | ✓ via `i32: Into<i32>` |
| raw `i32` | `find_by_id(7_u8)` | ✓ via `u8: Into<i32>` (pre-2.0 behavior preserved) |
| `CakePk = Id<E, i32>` | `find_by_id(CakePk::new(7))` | ✓ trivial `Into` |
| `CakePk` | `find_by_id(7_i32)` | ✗ no `From<i32> for Id<E, i32>` |
| `CakePk` | `find_by_id(post::PostPk::new(7))` | ✗ phantom types differ |

`DeleteMany::filter_by_ids` (plural, takes an `IntoIterator<Item =
ValueType>`) keeps the exact-type-per-item bound for backwards
compatibility; relaxing per-item bounds would be a behavior change
unrelated to this PR.

---

## What's in the PR

| Path | Purpose |
|---|---|
| `src/entity/id.rs` | `Id<E, T>`, `FindByIdArg`, blanket impls, safety docs |
| `src/entity/auto_increment_hint.rs` | `PkAutoIncrementHint` + builtin impls |
| `sea-orm-macros/src/derives/entity_model.rs` | Macro emits trait-call for `auto_increment()` body |
| `sea-orm-macros/src/derives/value_type.rs` | `DeriveValueType` emits `DelegatesPkAutoIncrementHint` |
| `sea-orm-cli/src/cli.rs` | `--with-pk-newtypes` flag |
| `sea-orm-codegen/src/entity/column.rs` | `Column::refs: Vec<ColumnRef>` + `PkNewtypeContext` + `get_rs_type` resolution chain |
| `sea-orm-codegen/src/entity/transformer.rs` | Populates `refs` from FK constraints (multi-parent supported) |
| `sea-orm-codegen/src/entity/writer.rs` | `PkNewtypeFormat`, `build_pk_newtype_index`, `build_role_wrapper_index`, `gen_pk_newtype_decls` |
| `src/entity/base_entity.rs` | `find_by_id`/`delete_by_id` use `FindByIdArg<Self>` bound |
| `src/entity/compound.rs` | `filter_by_id` uses `FindByIdArg<Self>` bound |
| `tests/value_type_pk_compile_fail/*.rs` | Trybuild compile-fail fixtures (raw int, wrong entity, self-ref role swap, cross-entity eq) |
| `tests/value_type_pk_safety_tests.rs` | Trybuild harness with substring directives, runs on CI |
| `tests/auto_increment_hint_tests.rs` | Unit tests for trait resolution (10 tests) |
| `examples/basic_typed_pk/` | End-to-end task-tracker example with real codegen output |

---

## CI integration

`examples/basic_typed_pk/` is added to the existing `examples`
matrix in `.github/workflows/rust.yml` — one line, no new job. Uses
the same `cargo test --manifest-path` invocation as every other
example.

The trybuild safety test runs in the unit-test job (and incidentally
in every per-database job that calls `cargo test --test '*'`). It
uses a hybrid approach:

1. Trybuild asserts must-fail (fixture compiling would fail the test).
2. Stderr exact-match against committed snapshots, which are
   generated against CI's stable rustc. Cosmetic drift fails the
   test loudly so contributors update the snapshots intentionally.
3. Substring directives (`// expect-error: <string>`) inside each
   fixture provide a third layer: the error must mention our
   `#[diagnostic::on_unimplemented]` message text, the trait name,
   or the offending type. These substrings are our own prose and
   stable across rustc versions.

When CI's stable rustc updates and stderr drift breaks the exact-
match, rebless workflow is one command:
`TRYBUILD=overwrite cargo test --test value_type_pk_safety_tests`.

---

## Out of scope

- A sum-type for multi-parent FK columns. Currently falls back to
  raw scalar; see "Deliberate fallback" above.
- Non-PK role disambiguation. Codegen only emits role wrappers for
  PK columns of junction tables; non-PK columns sharing a parent
  share the parent's alias.
- `DeleteMany::filter_by_ids` (plural) relaxation. Kept on its
  pre-PR `IntoIterator<Item = ValueType>` bound to avoid unrelated
  behavior changes.
