# langdex

The language table for source files and blocks of code.  
**A library with no command.** Import it as `langdex`.  
This file is the development contract.

## Scope

- One entry per language: its name, aliases, file extensions, file names and interpreters. A fence names a language by its name or an alias.
- The lookups: by name, by path, by the first line of a file, and by a fence info string.
- The generated table and the task that regenerates it from GitHub Linguist at a pinned commit.

A caller keeps its own rules, such as which languages it reads and what it does with each.

## Rules

1. **Data only.** No lexers, grammars or file reads: a lookup takes a name, a path or a line the caller passes in.
2. **One namespace for names and aliases**, matched without regard to case, as Linguist matches them.
   A name that matches nothing falls back to the extensions, so `mts` names TypeScript.
3. **An extension can belong to several languages.**
   Each extension lists its languages in a fixed priority order, and a lookup returns the first, or every one through the lookup that lists them all.
   The order is Linguist's popular languages first, then the languages whose first extension it is, then the name (operator, 2026-11-13).
   `data/local.yml` moves a language to the front where that order picks a niche one, such as GCC Machine Description for `.md`, and each entry gives its reason.
4. **The generated table is committed and never edited by hand.**
   One task regenerates it from the pinned Linguist commit, and a local entry lives in one file with the reason it exists.
5. **MIT OR Apache-2.0**, so permissive crates can depend on langdex. No dependency may carry a copyleft license.
6. **Nothing local leaks.** The repository is public: no local path, machine name, private repository or credential in any tracked file.

## Workflow

- Every change goes through a pull request, and the pull request's CI verifies it.
- Shared repository files are rendered from the shared Copier template; change them through a template update.
