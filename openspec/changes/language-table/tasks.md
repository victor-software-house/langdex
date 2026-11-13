# Tasks

Every compile, lint and test step runs in a box or the pull request's CI, not on the authoring machine.

## 1. Data

- [x] 1.1 Vendor `languages.yml` and `popular.yml` of Linguist `v9.7.0` under `data/linguist/`, with `COMMIT`, `SHA256SUMS` and `LICENSE-LINGUIST`, and add the `linguist:fetch` task.  
  Verify `mise run linguist:fetch` leaves the files unchanged.
  - 2026-11-13 (UTC): `mise run linguist:fetch` in a 4x8 box downloaded both files at the pinned commit; `shasum -a 256 -c SHA256SUMS` printed OK for each, and their hashes equal the committed ones.
- [x] 1.2 Write `data/local.yml` with the seven extensions of design section 1.  
  Verify the table test accepts each entry and refuses one that is not needed.
  - 2026-11-13 (UTC): the table test passes with the seven entries. It fails with a message naming the entry for `.rs` Rust (already first), `.zzz` (no language lists it) and `.c` Rust (does not list it).

## 2. Table and lookups

- [x] 2.1 Write the generator in `tests/table.rs` and the `generate` task, and commit the generated `src/table.rs`.  
  Verify the table test passes, and fails after a hand edit of `src/table.rs`.
  - 2026-11-13 (UTC): `mise run generate` wrote 833 languages, 1,486 extensions, 419 file names and 168 interpreters. The test passes, and fails at line 9 naming `mise run generate` after a hand edit. `cargo fmt --check` leaves the table alone.
- [x] 2.2 Write the types and lookups of design sections 4 and 5 in `src/lib.rs`, with rustdoc for each.  
  Verify `tests/lookups.rs` covers each scenario of the `language-table` spec and the order table of design section 2.
  - 2026-11-13 (UTC): `cargo test` passes 16 lookup tests and 12 doc tests. `cargo clippy --all-targets --all-features -- -D warnings` and `cargo doc --no-deps` pass.

## 3. Docs and release

- [ ] 3.1 Update `README.md` and `AGENTS.md`, and add the patch changeset for `0.0.1`.  
  Verify the prose lint and `openspec validate --all --strict` pass.
- [ ] 3.2 Verify the pull request's CI passes, and record the binary size of design section 7 in the pull request.
