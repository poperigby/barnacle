# SQL Branch Changes

This document explains the changes on the `sql` branch compared with the main
line of development. In this checkout the main branch equivalent is the
`trunk` bookmark, so the comparison used here is `trunk..sql`.

## Summary

The branch replaces Barnacle's old `agdb` graph-style persistence layer with a
SeaORM-backed SQLite schema. That is the central change. Most of the other
changes exist to make that migration compile and behave correctly:

- Repository initialization became asynchronous because SeaORM database access is
  async.
- Domain entities now store SQL row IDs instead of custom graph entity IDs.
- Parent-child relationships are now represented as foreign keys instead of graph
  edges.
- Active game/profile state moved from special graph nodes to boolean columns.
- Duplicate-name checks moved from manual pre-checks to SQL uniqueness
  constraints.
- The CLI and GUI were updated to await repository and entity operations.
- The GUI mod list gained a local table widget and cached row data so async
  entity lookups do not have to happen during view rendering.

The branch currently builds with `cargo check --workspace`.

## Commit-Level Intent

The branch contains these logical steps:

- `gui: table stuff`: adds a custom table widget and starts moving the mod list to
  tabular row rendering.
- `lib: create models for SeaORM`: introduces SeaORM entity models for the core
  data types.
- `Move stuff over to sql`: rewrites repository operations to use SQL queries and
  updates callers for async behavior.
- `barnacle-lib: make async`: makes the public repository/entity API async so it
  matches SeaORM.
- `gui: fix weird name passing`: stores display names in GUI row structs instead
  of calling async entity methods from synchronous view code.
- `Cleanup`: removes transitional code and simplifies call sites after the async
  conversion.
- `lib: clarify composite key`: documents and encodes uniqueness rules like "mod
  names are unique per game" and "profile names are unique per game."
- `lib: get rid of stupid domain error`: removes the custom duplicate-name domain
  error and relies on database errors for uniqueness violations.
- `lib: use auto entity register`: switches SeaORM schema setup to the automatic
  entity registry.

## Dependency Changes

### `barnacle-lib/Cargo.toml`

New dependencies were added:

- `sea-orm` with SQLite, Tokio runtime, macros, schema sync, and entity registry
  features.
- `serde_json`, used to serialize fields that do not yet map naturally to SQL
  columns, such as game targets.

The old `agdb` dependency is still present in `Cargo.toml`, but the repository
code no longer uses it in the main persistence path. This looks like leftover
dependency cleanup rather than an intentional runtime requirement.

### `barnacle-cli/Cargo.toml`

The CLI now depends on `tokio` because `Repository::new()` and most repository
operations are async.

### `Cargo.lock`

The lockfile changed heavily because SeaORM brings in SQLx, SQLite support,
schema-generation dependencies, and async database support crates.

## Database Layer

### `barnacle-lib/src/repository/db/mod.rs`

The database wrapper changed from:

- `agdb::DbAny`
- a `parking_lot::RwLock`
- synchronous setup
- manual graph aliases like `games`, `mods`, `active_game`, and `next_uid`

to:

- `sea_orm::DatabaseConnection`
- async initialization
- a SQLite URL pointing at `state_dir()/data.db`
- schema creation through SeaORM's entity registry
- an explicit `conn()` accessor for query code

Why this was made:

SeaORM needs an async connection and typed entity metadata. Instead of creating
graph aliases and manually linking nodes, the branch asks SeaORM to sync the SQL
schema from the registered entity models. This makes the schema explicit and lets
the ORM handle table creation, primary keys, foreign keys, and unique
constraints.

The old model-version, backup, and migration scaffolding was removed from the
active path. That old code was tied to the `agdb` graph layout. SeaORM schema
sync now handles the initial schema shape, although long-term data migrations
would still need a dedicated migration story.

## SeaORM Models

The files under `barnacle-lib/src/repository/db/models/` were rewritten from
`agdb` payload structs into SeaORM entity models.

### `games.rs`

`GameModel` became the SeaORM `games` table:

- `id` is the primary key.
- `name` is globally unique.
- `targets` is stored as a JSON string.
- `deploy_kind` is stored as a string.
- `is_active` stores the active-game flag directly on the row.

`DeployKind` gained `EnumString` so string values from the database can be parsed
back into the enum.

Why this was made:

The graph database previously stored game data as graph element values and active
state as an edge from a special `active_game` node. SQL needs columns and
constraints instead. Storing `is_active` on the game row makes the active-game
query a simple filtered lookup.

### `profiles.rs`

Profiles became a `profiles` table:

- `id` is the primary key.
- `game_id` points to the owning game.
- `name` is unique together with `game_id`.
- `is_active` stores active-profile state.
- The game relation uses cascade update/delete.

Why this was made:

Profiles are only required to be unique inside a game, not globally. The
composite unique key preserves that domain rule at the database level. The
foreign key replaces the old graph edge from game to profile.

### `mods.rs`

Mods became a `mods` table:

- `id` is the primary key.
- `game_id` points to the owning game.
- `name` is unique together with `game_id`.
- The game relation uses cascade update/delete.

Why this was made:

Like profiles, mod names only need to be unique within one game. The composite
unique key lets the database enforce that directly. The foreign key replaces the
old graph edge from game to mod.

### `mod_entries.rs`

Mod entries became a `mod_entries` table:

- `id` is the primary key.
- `profile_id` points to the profile that owns the entry.
- `mod_id` points to the referenced mod.
- `position` stores the entry order.
- `enabled` and `notes` are regular columns.
- Profile and mod relations cascade on update/delete.

Why this was made:

The old graph representation encoded mod-entry ordering by linking entries
together with edges. SQL does not need that linked-list shape. A numeric
`position` column is easier to query, sort, update, and enforce.

One implementation detail to note: the current model marks `position` as unique
under the `profile_position` key, but only `position` participates in that key.
The intent appears to be "position is unique per profile"; if so, `profile_id`
should also be part of the same unique key.

### `tools.rs`

Tools became a `tools` table:

- `id` is the primary key.
- `game_id` points to the owning game.
- `name`, `path`, and optional `args` are regular columns.

Why this was made:

Tools are now modeled consistently with the rest of the SQL schema. Paths are
stored as strings because SQLite has no native `PathBuf` type.

## Entity API Changes

The domain entity types in `barnacle-lib/src/repository/entities/` were rewritten
around SQL rows.

### Entity IDs

`entity_id.rs` was deleted. The old `EntityId` wrapper combined an `agdb::DbId`
with a generated UID so stale graph IDs could be detected. With SQL rows, each
entity now stores an `i64` primary key directly.

Why this was made:

SQL primary keys and row lookups replace the graph ID plus UID mechanism. Stale
handles are detected by reloading the row and returning `RemovedEntity` if no row
exists.

### Error Handling

The entity error type now wraps `sea_orm::DbErr` instead of `agdb::DbError`.
`DuplicateName` was removed, and tests now expect SeaORM/SQLite unique
constraint errors for duplicate inserts.

Why this was made:

Uniqueness is now enforced by SQL constraints. That removes pre-insert scans and
avoids race-prone "check then insert" logic. The tradeoff is that callers now see
database-flavored duplicate errors unless a new domain-level mapping is added
later.

### `Game`

Game operations are now async and query the `games`, `profiles`, and `mods`
tables through SeaORM.

Important changes:

- `Game::load` fetches by row ID and errors with `RemovedEntity` if missing.
- `name`, `targets`, `deploy_kind`, `dir`, `set_name`, and `set_deploy_kind` are
  async.
- `add`, `list`, `search`, `active`, and `activate` are SQL queries.
- Active game state is managed by setting all games' `is_active` to false, then
  setting the selected game to true.
- Removing a game removes child profiles and mods first, deletes the row, removes
  the game directory, and activates the next available game if needed.

Why this was made:

These changes translate graph traversal and graph mutation into relational
queries. The active-state rewrite removes special graph nodes and makes active
selection explicit in the `games` table.

### `Profile`

Profiles are now loaded and queried from the `profiles` table.

Important changes:

- Profile methods are async.
- Parent lookup reads `game_id` and loads the parent `Game`.
- Profile listing filters by `game_id`.
- Profile search filters by both `game_id` and `name`.
- Active profile state is stored in `is_active`.
- Removing a profile deletes its mod entries, deletes the row, removes its
  directory if present, and activates the next profile if the removed one was
  active.

Why this was made:

The SQL schema expresses profile ownership with `game_id`, so all profile
operations now use that foreign key instead of graph traversal.

### `Mod`

Mods are now loaded and queried from the `mods` table.

Important changes:

- Mod methods are async.
- Parent lookup reads `game_id`.
- Adding a mod inserts a row into `mods`, then creates or extracts the on-disk mod
  directory.
- Removing a mod deletes the row and removes its directory if present.

Why this was made:

The branch keeps the existing filesystem behavior but changes the persisted
identity and parent relationship to SQL rows and foreign keys.

### `ModEntry`

Mod entries are now loaded and queried from the `mod_entries` table.

Important changes:

- `name()` reads through the referenced mod row.
- `enabled()`, `set_enabled()`, and `notes()` read/write columns on the entry row.
- `parent()` reads `profile_id`.
- Adding an entry appends it using `position = current_entry_count`.
- Listing entries orders by `position`, then `id`.
- Removing an entry deletes the row and decrements positions for later entries in
  the same profile.

Why this was made:

The old graph implementation represented load order through graph edges. The SQL
version makes ordering a first-class column, which is much easier to sort and
repair after deletion.

### `Tool`

Tools were updated to load from the `tools` table and expose async getters for
`name`, `path`, and `args`.

Why this was made:

This keeps tools aligned with the rest of the SeaORM model conversion, even
though tool usage appears less developed than games, profiles, mods, and mod
entries.

## Repository API

`Repository::new()` and `Repository::mock()` are now async. The high-level
methods such as `add_game`, `games`, `search_game`, and `active_game` await the
corresponding entity operations.

The old `Default` implementation was removed because constructing a repository
now requires async database initialization.

Why this was made:

SeaORM's connection and query APIs are async. A synchronous `Default` constructor
would either block internally or require a hidden runtime, so the branch makes
async initialization explicit.

## CLI Changes

The CLI now runs under `#[tokio::main]`.

Updated behavior:

- `Repository::new().await` initializes the database.
- `game`, `profile`, and `mod` command handlers are async.
- Calls like `repo.games()`, `repo.active_game()`, `game.name()`,
  `profile.activate()`, and `active_profile.mod_entries()` are awaited.

Why this was made:

The CLI is a direct caller of the repository API. Once the library became async,
the CLI needed a Tokio runtime and async command handlers.

## GUI Changes

The GUI was updated to work with the async repository API and to avoid async
calls during view rendering.

### Runtime Initialization

`App::new()` creates a small Tokio runtime and uses it to block on
`Repository::new()`.

Why this was made:

Iced's application constructor is synchronous, but the repository constructor is
now async. The local runtime bridges that mismatch during startup.

### Async Tasks Instead of `spawn_blocking`

Several places that previously wrapped synchronous repository calls in
`spawn_blocking` now use async `Task::perform` bodies directly.

Why this was made:

The database operations are no longer blocking synchronous work. Running them as
async tasks is a better fit for SeaORM and avoids unnecessary blocking-thread
handoffs.

### Cached GUI Rows

The library manager, profile tab, and mod list now build row structs that contain
both the entity handle and already-loaded display fields:

- `GameRow`
- `ProfileRow`
- `ModEntryRow`
- `ProfileOption`

Why this was made:

Iced view functions are synchronous. After names and enabled states became async
database reads, the UI could no longer call `entity.name()` or `entry.enabled()`
while building widgets. Caching those values during async load tasks keeps views
synchronous and predictable.

### Custom Table Widget

`barnacle-gui/src/widgets/table.rs` adds a reusable table widget with:

- column definitions
- headers
- row-to-cell rendering closures
- configurable width, padding, and separators
- custom layout and draw logic

`barnacle-gui/src/widgets/mod.rs` exports the widget, and the mod list uses it
for the mod table.

Why this was made:

The mod list needs a structured table with sortable headers and per-row controls
like enabled checkboxes. The custom widget gives the GUI control over layout and
styling instead of relying on ad hoc rows.

### Mod List Updates

The mod list now renders `ModEntryRow` values instead of raw `ModEntry` handles.
Toggling a mod entry runs `set_enabled().await`, then patches the cached row
state when the task completes.

Why this was made:

This keeps the UI responsive and avoids refreshing the whole list for a simple
checkbox toggle.

## Configuration and Development Shell

`nix/shell.nix` gained helper scripts:

- `rmshare`: removes Barnacle's local share directory.
- `rmdb`: removes Barnacle's local state directory, including the SQLite DB.
- `nuke`: removes local share, state, and config directories.

Why this was made:

Database schema work often needs a clean local application state. These helpers
make it faster to reset test data while iterating on the SQL migration.

## Documentation

`barnacle-lib/CONTRIBUTING.md` changed, mainly to reflect the new async testing
and repository setup patterns.

Why this was made:

Tests and examples that construct repositories now need async setup, so
contributor documentation needed to match the new API shape.

## Behavioral Changes

The most important user-visible and developer-visible behavior changes are:

- Database storage is now relational SQLite via SeaORM instead of an `agdb` graph.
- A new database is schema-synced from SeaORM entity definitions at startup.
- Entity methods that touch the database are async.
- Games are sorted by name when listed.
- Mods and profiles are listed by SQL query order rather than graph traversal.
- Mod entries are ordered by explicit numeric position.
- Removing a mod entry compacts later positions.
- Duplicate game names are rejected by a global unique constraint.
- Duplicate profile names are rejected within the same game.
- Duplicate mod names are rejected within the same game.
- Active game/profile state is stored on rows instead of in special graph edges.
- Stale entity handles produce `RemovedEntity` when their row is missing.

## Why The Branch Exists

The branch appears to be motivated by replacing a hand-managed graph persistence
model with a more conventional relational database layer. The practical benefits
are:

- clearer schema definitions;
- database-enforced uniqueness;
- simpler parent-child lookups through foreign keys;
- less custom ID and stale-handle machinery;
- easier ordering for mod entries;
- better alignment with async Rust database tooling;
- a cleaner path toward richer queries and future migrations.

The cost is that async database access now propagates through the library, CLI,
GUI, and tests. The branch makes those call sites async and adjusts the GUI so
async data is loaded before rendering.

## Verification

Ran:

```sh
cargo check --workspace
```

Result:

- Build passed.
- One warning remains in `barnacle-gui`: `CURRENT_CONFIG_VERSION` is unused.
