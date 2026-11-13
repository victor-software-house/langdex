# Proposal

## Why

hakkiri and nishiki each keep their own lists of file extensions, and neither knows that `md` and `markdown` name one language, or that `app.mts` is TypeScript.  
A scope or a fence that names a language by an alias misses files, and a rule scoped to `txt` reached `.rst` files in hakkiri.  
GitHub Linguist keeps the names, aliases, extensions, file names and interpreters of 833 languages, under the MIT license.  
langdex turns that data into a table and lookups that any permissive crate can use, with no runtime dependency and no network at build time.

## What Changes

- The Linguist data of tag `v9.7.0` is vendored under `data/linguist/`, with its license and the sha256 of each file.
- A generator, run as a test, writes `src/table.rs` from that data and a local file, and the same test fails CI when the committed table differs from what it would write.
- An extension shared by several languages lists them in a fixed order: Linguist's popular languages first, then the languages whose first extension it is, then by name. A local file sets the first language for the extensions where that order picks a niche language, each with its reason (operator, 2026-11-13).
- The lookups: by name or alias, by extension, by file name, by path, by interpreter, by the first line of a file and by a fence info string. A name that matches nothing falls back to the extensions.
- The first release, `0.0.1`, publishes the crate to crates.io.

## Capabilities

### New Capabilities

- `language-table`: the table and its lookups.

## Impact

- New: `data/`, `src/table.rs`, `tests/table.rs`, `tests/lookups.rs`, `LICENSE-LINGUIST`.
- Changed: `src/lib.rs`, `Cargo.toml` (the files the crate packs, two dev-dependencies), `mise.toml` (the generate and fetch tasks), `README.md`, `AGENTS.md`.
- The crate has no dependency. Its tests use `vsh-serde-saphyr` and `serde`, both MIT OR Apache-2.0.
