# Rust Coding Style

Rust code in this repo (`engine/src/`, `engine/crates/*`) follows the rules below. Edition 2024, latest stable toolchain. These rules complement `engine/CLAUDE.md` and `bevy-patterns.md`.

These rules apply to code you write or change. Code that predates a rule is brought in line when it is touched; do not rewrite untouched files only to comply.

## Formatting & Linting

- Use `cargo fmt` (default rustfmt settings, no rustfmt.toml).
- Use `cargo clippy -- -D warnings`. The `type_complexity` and `result_large_err` lints are allowed workspace-wide.
- Run `make fix-lint` to apply both: `cargo clippy --workspace --fix --allow-dirty && cargo fmt --all`.

## Module layout — no `mod.rs`

Required:

| Pattern                 | Example                                       | Why                                                                            |
| ----------------------- | --------------------------------------------- | ------------------------------------------------------------------------------ |
| Rust 2018+ module files | `route/persona.rs` + `route/persona/create.rs` | One file declares the module; no ambiguity about which file is the module root |

Forbidden:

| Pattern                 | Example                  | Why                                                                          |
| ----------------------- | ------------------------ | ---------------------------------------------------------------------------- |
| `mod.rs` as module root | `route/persona/mod.rs`   | Hard to navigate (many files all named `mod.rs`); editor tabs look identical |

## Naming

- Crates: `snake_case` prefixed with `homunculus_` (e.g., `homunculus_core`, `homunculus_api`).
- Types/structs/enums: `PascalCase` (e.g., `VrmApi`, `HttpState`).
- Functions/methods: `snake_case` (e.g., `list_expressions`, `set_expressions`).
- Constants: `SCREAMING_SNAKE_CASE`.

### Query parameters

Bevy `Query` system parameters must not use a `_q` suffix. Use a descriptive noun instead:

- Singular (`window`, `camera`) when the query is expected to return one result and the system calls `.single()` / `.single_mut()`.
- Plural (`windows`, `personas`) when the query is iterated or used with `.get()` over an arbitrary entity.

Forbidden:

| Pattern                              | Example                       | Fix                         |
| ------------------------------------ | ----------------------------- | --------------------------- |
| `_q` suffix on any `Query` parameter | `window_q: Query<&Window, …>` | `window: Query<&Window, …>` |

## Constructors — type-building functions are associated functions

A function whose job is to build a value of a struct or enum defined in
this crate belongs on that type as an associated function
(`impl T { fn … -> Self }`), called as `T::build(…)`, not as a free
`fn build_t(…) -> T`. Co-locating the constructor with its type keeps
every way to produce a `T` reachable from `T` itself, and reads as
`AssetId::parse(…)` at the call site rather than a bare
`parse_asset_id(…)` whose return type you have to infer from the name.

This covers factory / parser / classifier / decoder helpers — any
function that assembles a fresh value of a local type from its inputs —
whether it returns `T`, `Option<T>`, or `Result<T, _>`.

Required:

| Instead of (free function)                               | Use (associated function)                                    |
| -------------------------------------------------------- | ------------------------------------------------------------ |
| `fn resolve_tray_position(s: &str) -> TrayPosition`      | `impl TrayPosition { fn resolve(s: &str) -> Self }`          |
| `fn parse_asset_id(s: &str) -> Option<AssetId>`          | `impl AssetId { fn parse(s: &str) -> Option<Self> }`         |
| `fn build_registry(mods: &ModRegistry) -> TrayMenuRegistry` | `impl TrayMenuRegistry { fn build(mods: &ModRegistry) -> Self }` |

- Return `-> Self` (not the spelled-out type name) and build variants
  with `Self::Variant` inside the `impl`.
- Name the method for what it does, dropping the type name the `T::`
  prefix already carries: `parse_asset_id` → `AssetId::parse`,
  `build_registry` → `TrayMenuRegistry::build`.

Required — a constructor that takes no argument is `Default`:

| Instead of (inherent no-arg constructor) | Use                                                         |
| ---------------------------------------- | ----------------------------------------------------------- |
| `impl ModRegistry { fn new() -> Self }`  | `impl Default for ModRegistry`, called as `ModRegistry::default()` |

- Write `#[derive(Default)]` when every field's default is the value the
  empty `T` needs.
- On a generic type, write the impl by hand (`impl<P> Default for
  Registry<P>`): the derive adds a `P: Default` bound the type parameter
  does not need.

Forbidden:

| Pattern                                                 | Why                                                                |
| ------------------------------------------------------- | ------------------------------------------------------------------ |
| Free `fn` returning a locally-defined `T` it constructs | The constructor is discoverable only by grep, not from `T`         |
| Inherent `fn new() -> Self` taking no argument          | `Default` is the standard spelling of an argument-free constructor |

Exceptions:

- **Foreign return type.** You cannot add an inherent method to a type
  defined in another crate (e.g. `bevy_tray_icon`'s `Menu`). Prefer a
  `From` / `TryFrom` / `FromStr` trait `impl` (those satisfy this rule —
  they are associated functions); fall back to a free `fn` only when no
  conversion trait fits.
- **Standard conversion traits.** `From` / `TryFrom` / `FromStr` /
  `Default` impls already are the idiomatic constructors — a free `fn`
  that should be one of these is converted to the trait, not to an
  inherent method.

## Serde & JSON

- All HTTP request/response structs MUST use `#[serde(rename_all = "camelCase")]`.
- Derive order: `Serialize, Deserialize, Debug, Clone` (in that order).
- For Rust reserved keywords as field names, use `#[serde(rename = "loop")]`.
- Use default value helper functions: `fn default_volume() -> f64 { 1.0 }`.

## Error Handling

- Use the project's `ApiResult<T>` type alias (`Result<T, ApiError>`) for all API operations.
- Use `thiserror::Error` for custom error enums.
- Map errors to HTTP status codes via `impl IntoResponse for ApiError`.
- Use `.output_log_if_error()` for broadcast/channel send failures.
- Never use `.unwrap()` in production code; use `?` or explicit error handling.

## Imports

Required:

- Every `use` is declared at the top of the file (immediately after the `//!` if present).
- All `use` statements form a single contiguous block — no blank lines separating `std`, external crates, and crate-local imports. rustfmt sorts the block.
- Use the Bevy prelude: `use bevy::prelude::*;`.
- Use crate-level imports: `use crate::error::ApiResult;`.

Exception:

- Inside `#[cfg(test)] mod tests { ... }` blocks, locally-scoped `use` statements are allowed (e.g., `use super::*;`, test-only fixtures). Test code is the only place where locality outweighs the "all imports at the top" rule.

Forbidden:

| Pattern                                             | Example                            | Why                              |
| --------------------------------------------------- | ---------------------------------- | -------------------------------- |
| `use` inside non-test functions or blocks           | `fn f() { use std::io; ... }`      | Spreads scope across the file    |
| Blank lines between `std` / external / crate `use`s | `use std::...;\n\nuse bevy::...;`  | We do not group                  |
| Glob import in consumer code                        | `use homunculus_core::components::*;` | Hides which symbols are in scope |

Note on preludes: a glob import of a crate's `prelude` module (`use bevy::prelude::*;`, `use homunculus_core::prelude::*;`) is allowed — curating those names is the prelude's purpose. A module that _defines_ a prelude may itself use `pub use foo::*;` inside its own definition. The rule above forbids every other glob import in **consumer** code.

Required — import, don't inline:

- Prefer `use` imports over inline fully-qualified paths. If a type is used in a function signature or system body, it belongs in the `use` block at the top of the file, not written out inline as `some_crate::some_module::Type` at the call site.

| Pattern                                                               | Example                                         | Fix                                                                       |
| --------------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------------------------- |
| Inline path in signature / body                                       | `fn f(x: homunculus_core::prelude::ModInfo)`    | Add `use homunculus_core::prelude::ModInfo;` and write `fn f(x: ModInfo)` |
| Inline path in `run_if` or type parameter                             | `.add_message::<bevy::window::WindowResized>()` | `use bevy::window::WindowResized;` then `.add_message::<WindowResized>()` |
| Inline path in `.after()` / `.before()` / `.in_set()` system-ordering | `.in_set(homunculus_core::system_set::HomunculusSystemSet::ScriptEventHandle)` | `use homunculus_core::system_set::HomunculusSystemSet;` then `.in_set(HomunculusSystemSet::ScriptEventHandle)` |
| Inline path in a macro call                                           | `tracing::warn!(…)`                             | `use tracing::warn;` then `warn!(…)`                                      |

## Item Ordering

Arrange items top-down within a file. High-level components go at the top; lower-level details go toward the bottom.

### File-level ordering

1. Module declarations (`mod` / `pub mod`) and re-exports (`pub use`)
2. Imports (`use`)
3. Constants and type definitions (`const`, `static`, `struct`, `enum`, `type`)
4. Entry points (`main`, `Plugin` struct + `impl Plugin`, `api!` macro invocations)
5. Public functions and method implementations (`pub fn`, `pub async fn`)
6. Crate-internal functions (`pub(crate) fn`)
7. Private functions and helpers (`fn`)
8. Tests (`#[cfg(test)]`)

### Ordering within impl blocks

1. `pub` methods
2. `pub(crate)` methods
3. Private methods

### Principles

- **Callers above, callees below.** `main` → functions it calls → their sub-functions.
- **Place struct/enum definitions above the impl blocks that use them.** Group type definitions near the top of the file.
- **Bevy Plugins are entry points.** Place Plugin struct + `impl Plugin` above the system functions registered in `build()`.

### Exceptions

- `macro_rules!` macro definitions are exempt from ordering rules.

## Function Granularity

- Extract functions at a granularity where the calling code reads naturally as prose. The caller states "what" to do; the helper handles "how".
- Aim for function bodies under 20 lines. If a function exceeds this, look for a named sub-operation to extract.
- Inline closures passed to combinators (`map`, `and_then`, `match` arms, etc.) that exceed 3 lines should be extracted as named functions.

## Comments

Non-doc line comments are restricted. The only permitted forms:

| Form                | Use                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------ |
| `// TODO: <text>`   | Work to address later                                                                |
| `// NOTE: <text>`   | A **critical caveat** — only when overlooking it causes real harm; see the bar below |
| `// SAFETY: <text>` | Required justification for any `unsafe { ... }` block (rustc / clippy idiom)         |

Forbidden:

| Pattern                                                  | Example                                   | Why                                                                 |
| -------------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------------- |
| Plain narrative comments                                 | `// increments counter`                   | What the code does belongs in identifiers                           |
| Region-divider comments                                  | `// ----------- Section Name -----------` | Use the file's natural structure (modules, impl blocks) instead     |
| Block comments                                           | `/* ... */`                               | Same as narrative comments                                          |
| Commented-out code                                       | `// let x = old_impl();`                  | History lives in git                                                |
| `// NOTE:` for merely non-obvious or "good to know" info | `// NOTE: this is the handler`            | NOTE is for critical caveats only — rename the identifier or delete |

A `// NOTE:` is reserved for a **critical caveat**: something that, if a
reader overlooks it, leads to a bug, a crash, data loss, a security
issue, or a violated invariant. "Non-obvious" or "good to know" is not
enough — the test is concrete harm on the line that misses it.
Qualifying examples: a race condition, a workaround for a specific
upstream bug, an ordering requirement the surrounding code silently
relies on (e.g. a system that must write `AppExit` before another
system reads it), an invariant a later mutation must preserve. If
overlooking the comment causes no real failure, do not write it —
rename an identifier so the code carries the meaning, or delete it.

Note: `///` and `//!` are **doc comments**, not "line comments" for this rule — see "Doc comments" and "Doc comment content — only what the reader needs" below.

## Comment references — never cite `docs/`

Comments and doc comments must not cite any document under `docs/` —
not by path, and not by section number or decision ID (`spec §7`,
`D17a of the migration design`).

Everything in `docs/` is short-lived by design, moves around, or stops
describing the code. `docs/superpowers/` and `docs/plans/` are not even
tracked in git, and the pages of the user guide under `docs/website/`
are renamed and merged as the guide is reorganized, so a comment that
points at one becomes a dead reference the next reader cannot follow —
and unlike a broken code path, nothing fails to compile when it does.

Normative external specifications and manuals — the VRM 1.0 / VRMA
specifications, glTF 2.0, the MCP specification, RFCs, platform API
references — may be cited by name and section, ideally with a short
quote so the citation stays checkable.

| Pattern                                       | Example                                    | Instead                                                                      |
| --------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------- |
| Citing a design doc or plan by path           | `// see docs/superpowers/specs/…-design.md` | State the decision the doc records (in a doc comment, only as the contract it implies), not a pointer to it |
| Citing a design doc by section or decision ID | `/// … (spec §7)` / `// … (D16)`           | Drop the pointer; state the contract (doc) or the caveat (`// NOTE:`) itself |
| Citing a user-guide page                      | `/// See docs/website/docs/mods/menu.md`   | State the behavior the page describes as the item's contract                 |
| Citing a normative specification              | `/// <spec name> § <section> — "<quote>"`  | **Allowed** — external specifications are permanent                          |

The rule is one-directional: Markdown under `docs/`, PR descriptions and
commit messages may link wherever they like. It binds only what ships
inside a `.rs` file.

When a working note holds reasoning a future reader will want, copy the
**conclusion** into the comment rather than linking the note. In a doc
comment the conclusion is copied only as the contract it implies; the
rationale behind it is not copied (see "Doc comment content — only what
the reader needs").

## Doc comments

Required:

| Place                                                                                      | Style                                               |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------- |
| Every externally-public item (`pub` only — not `pub(crate)`, `pub(super)`, `pub(in path)`) | `///` — one-line summary, blank line, optional body |
| Each file-level module — `lib.rs`, `main.rs`, and every `foo.rs` that declares a module    | `//!` — module-level purpose in 1–3 lines           |

Equivalent attribute forms (`#[doc = "..."]`, `#[doc = include_str!("README.md")]`) count as doc comments and satisfy this rule.

Not required (but recommended):

- `pub(crate)` / `pub(super)` / `pub(in path)` items
- Inline modules (`mod inner { ... }` inside another file)
- `#[cfg(test)] mod tests { ... }` blocks and their contents — EXCEPT
  `#[test]` functions themselves, which are covered by "Test doc
  comments" below

Style guide:

- The first line is a noun phrase or third-person singular verb phrase (e.g., `/// Returns the persona linked to this webview.`). Stay descriptive, not imperative.
- Code examples use triple backticks under an `# Examples` heading; hidden setup lines may be prefixed with `# `. Complex APIs should carry one.
- Invariants live under a `# Invariants` section.

Forbidden:

| Pattern                                     | Why                                       |
| ------------------------------------------- | ----------------------------------------- |
| Externally `pub` item with no doc           | Public API owes the reader an explanation |
| Placeholder doc like `/// TODO: write this` | Don't ship empty docs                     |

## Doc comment content — only what the reader needs

A doc comment (`///`, `//!`, or the equivalent `#[doc = "…"]`) states the
contract a reader relies on to use the item, and nothing else. Content
outside the allowed list below is deleted — not moved to a `//` comment, a
`# Notes` section, or another item's doc — except as the `NOTE:` boundary
case below allows. This applies to every doc comment under `engine/src/`
and `engine/crates/`, whatever the item's visibility.

Allowed:

| Content | Example |
| ------- | ------- |
| The one-line summary | `/// Returns the persona linked to this webview.` |
| Observable behavior, and what the return value means — including `None`, `false`, and sentinel values | `` Returns the persona's entity, or `None` when no persona with that ID is spawned. `` |
| Meaning of an input that its type does not state: unit, coordinate space, base, valid range | `A screen position in logical pixels, with the origin at the top-left of the primary monitor.` |
| Side effects and state changes the caller can observe | `The persona's current animation keeps playing.` |
| Preconditions and caller obligations, including an ordering constraint whose violation breaks behavior | `Must run before any mod service is spawned.` |
| Edge cases and boundary values | `An empty list clears every expression.` |
| `# Panics`, `# Errors`, `# Safety` | |
| `# Invariants` — the guarantee itself | `Persona IDs are never reused within one database.` |
| Limitations and unsupported cases the caller can observe, stated in the present tense | `Only VRM 1.0 models load; VRM 0.x files are rejected.` |
| Spec mapping: `# References`, and citations of normative specifications (VRM 1.0 / VRMA, glTF 2.0, MCP, RFCs) | `<spec name> § <section> — "<quote>"` |
| For a `#[test]` fn: the asserted contract and the `Case:` scenario (see "Test doc comments") | |
| `# Examples` | |
| `TODO: <text>` | `TODO: support VRM 0.x models.` |

Forbidden (representative — the allowed list above is authoritative):

| Category | Example |
| -------- | ------- |
| Implementation steps — how the body does its work | `Personas are therefore resolved through an ID-keyed map rather than by entity.` |
| Design rationale and rejected alternatives, including why an item is named or placed as it is | `we use the name Persona instead because …` |
| External history and comparisons with other implementations: other apps' behavior, issues, commits, dependency source lines | `bevy issue #12345` / `the trade UniVRM makes too` |
| Hypothetical breakage, and deferral to another item's doc | `A rewrite that renumbers from zero would silently …` / `` [`ModRegistry::register`] records why `` |
| Time-bound wording, and pointers to design documents the reader cannot follow | `Linux support arrives later.` / `for now` / `until Phase 3` / `(spec §7)` / `D17a of the migration design` |
| Echo of the code: the name, signature, or item kind restated; ECS registration, schedule, or `run_if` recited; another item's doc copied | `Bevy Resource wrapping …` / `Startup system that …` / `Runs in PostUpdate before …` |
| Callers and tests, in a non-test item's doc: who calls the item, how testable it is, what the tests cover | `Used by the tray plugin.` / `Kept pure so unit tests can cover every branch.` |
| Absence inventory: what the item does not hold or know, when no caller would expect it to | `The registry knows nothing about webviews.` |
| A `NOTE:` tag inside a doc comment | `/// NOTE: …` |

Boundary cases:

- An intra-doc link that names a type or item the contract uses is allowed;
  a link that hands the explanation to another item is not.
- "Preconditions and caller obligations" covers what a caller must do;
  "Callers and tests" covers who the callers are. Only the first is allowed.
- An `# Invariants` entry states the guarantee; what would break without it
  is not written.
- A contrast that states what the item does against a behavior a caller
  could expect ("truncating rather than wrapping", "ignored rather than
  clamped") is observable behavior and is allowed; an alternative the
  designers considered and rejected is not.
- Citing what a specification says ("the VRM 1.0 specification defines
  …") is spec mapping; describing what another implementation does
  ("UniVRM clamps the weight …") is a comparison.
- Schedule and ordering information is allowed on an item an outside system
  must order against (a `SystemSet` or `ScheduleLabel`, for instance);
  reciting where the item itself is registered or scheduled is not.
- Each item states its own contract, even when it matches a delegate's;
  "another item's doc copied" means text that describes a different item.
- A sentence that mixes allowed and forbidden content is split: the allowed
  part stays, and a consequence of misuse is restated as the precondition or
  `# Panics` entry it implies.
- Future work is written only as a `TODO:`, whose text the time-bound ban
  does not cover. Untagged future-design prose becomes a `TODO:` when it
  names pending work and is deleted otherwise.
- A heading outside the allowed list (`# Notes`, `# Layout`, `# Usage`, …)
  grants no allowance: each sentence under it is judged by the rows above.
  Usage examples go under `# Examples`.
- A module doc (`//!`) states the module's purpose in 1–3 lines; it does not
  list the module's items, files, or callers.
- A field or variant doc does not repeat its parent's doc or a sibling's.
- A `NOTE:` found in a doc comment is rewritten as allowed doc content when
  it states a contract, becomes a body `// NOTE:` when it meets the bar in
  "Comments", and is deleted otherwise.
- Test doc comments keep the two-part structure in "Test doc comments"; the
  forbidden list applies to them as well.

## Comment prose — write complete English sentences

English prose in comment and doc bodies — doc-comment paragraphs
(including `Case:` paragraphs) and the text of `// TODO:` / `// NOTE:` /
`// SAFETY:` comments — is written as grammatically complete, natural
sentences. Telegraphic fragments that drop subjects, verbs, or articles
read as shorthand notes, not documentation, and force the reader to
reconstruct the elided grammar.

Required:

| Instead of (fragment)                                       | Use (complete sentence)                                                                                       |
| ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `Case: the drag path — press grabs, move follows, release drops.` | `Case: the user presses on a character to grab it, moves the pointer to drag it, and releases it to drop it.` |
| `Case: the click-outside teardown.`                         | `Case: the user clicks outside the context menu to dismiss it.`                                               |

Forbidden:

| Pattern                                                   | Example                     | Why                                          |
| --------------------------------------------------------- | --------------------------- | -------------------------------------------- |
| Noun-phrase-only sentence in a body paragraph             | `Case: the drag path.`      | A sentence needs a verb; say what happens    |
| Clause chains with dropped subjects, objects, or articles | `press grabs, move follows` | Shorthand the reader must mentally re-expand |

Not fragments for this rule (still fine):

- The **first line** of a doc comment — the style guide above
  deliberately makes it a noun phrase or third-person singular verb
  phrase (`/// Returns the persona linked to this webview.`).
- Standard parallel ellipsis sharing an auxiliary or subject
  (`are clamped to the monitor, not wrapped`).
- Em-dashes, semicolons, and colons joining clauses that are each
  complete on their own.
- Table cells, section headings, and list labels inside doc bodies.

Complete does not mean long — keep comments concise:

- Say it once: one to three short sentences per paragraph; a doc body
  that can be one sentence stays one sentence.
- Cut filler and repetition: do not restate the first line in the
  body. What a doc comment may contain at all is set by "Doc comment
  content — only what the reader needs".
- When trimming, drop whole sentences rather than degrading the
  survivors into fragments — the complete-sentences rule above still
  applies to what remains.

## Test doc comments

Every `#[test]` function carries a `///` doc comment that states the
asserted contract AND the concrete case the test envisions:

| Place                    | Style                                                                                                                       |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| Every `#[test]` function | `///` — first line: what the test asserts; blank line; a `Case:` paragraph naming the envisioned scenario — and nothing else |

- The `Case:` paragraph describes the real-world scenario the test
  envisions — the user action and the app state it happens in. It is
  NOT a restatement of the assertions; a reader should learn which
  situation the contract serves, not what the `assert_eq!` lines
  already say.
- A test doc holds exactly those two parts — no policy paragraph in
  between. When a test pins a decided policy (e.g. "a tray item without
  a command is not registered"), fold the decision — including the
  behavior a reader might expect instead, when it fits — into the first
  line ("Asserts that a submenu container is skipped rather than
  registered."); the rationale behind the decision is not written in
  any doc comment.
- Keep the `Case:` paragraph to the scenario and stop — 1–3 sentences.
  The scenario is the paragraph's ONLY content; in particular:
  - Do not restate what the test itself pins — the first line already
    says it.
  - Do not state policies, design decisions, or their rationale in the
    `Case:` paragraph — fold the decision into the first line and
    leave the rationale out.
  - Do not speculate about how a hypothetical broken implementation
    would misbehave ("a registry that keeps containers would run an
    empty command") — that is the test's justification, not the case.
  - Do not add layering footnotes about contracts pinned by another
    layer's tests — leave them out entirely rather than pointing at
    them.

```rust
/// Asserts that a submenu container is skipped rather than registered,
/// while each of its leaf items is registered under its own ID.
///
/// Case: a mod declares a "Tools" submenu in its tray entry, and only the
/// items inside it carry commands.
#[test]
fn submenu_container_is_skipped_but_children_are_registered() { ... }
```

Not required:

- Test helpers, fixtures, and mocks inside `#[cfg(test)]` — keep their
  names descriptive instead (a doc comment is still welcome when it
  states a non-obvious contract of the helper, e.g. how many mods the
  seeded registry holds).

## Edition & Workspace

- Rust edition 2024.
- Workspace version: see `version.toml` at repo root.
- License: MIT OR Apache-2.0.
- All new crates must be added to the workspace `Cargo.toml`.
- New crate `[package]` fields must use workspace inheritance:
  ```toml
  version.workspace = true
  edition.workspace = true
  authors.workspace = true
  repository.workspace = true
  license.workspace = true
  publish.workspace = true
  ```
- Dependencies must use `workspace = true` when available in `[workspace.dependencies]`. Add new shared dependencies to the workspace before referencing them.
