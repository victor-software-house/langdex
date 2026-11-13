## ADDED Requirements

### Requirement: A name or alias names one language

`by_name` SHALL find a language by its name, its `fs_name` or one of its aliases, without regard to case.  
A name that matches none SHALL be read as an extension.

#### Scenario: Names of Markdown

- **WHEN** a caller looks up `md`, `markdown` and `Markdown`
- **THEN** each gives Markdown

#### Scenario: An extension as a name

- **WHEN** a caller looks up the name `mts`
- **THEN** it gives TypeScript, whose extensions hold `.mts`

### Requirement: A shared extension lists its languages in one order

An extension that several languages list SHALL give them popular first, then those whose first extension it is, then by name, and `data/local.yml` SHALL move its language to the front.  
A lookup that returns one language SHALL return the first.

#### Scenario: A header file

- **WHEN** a caller looks up every language of `.h`
- **THEN** it gives C, C++ and Objective-C, in that order

#### Scenario: A Markdown file

- **WHEN** a caller looks up the path `docs/a.md`
- **THEN** it gives Markdown, as `data/local.yml` sets

### Requirement: A path finds its language by file name, then extension

`by_path` SHALL read the file name of a path first, then its extensions from the longest suffix.

#### Scenario: Files by name and extension

- **WHEN** a caller looks up `Makefile`, `app.mjs` and `app.mts`
- **THEN** they give Makefile, JavaScript and TypeScript

### Requirement: A first line finds its language by interpreter

`by_first_line` SHALL read the interpreter of a shebang line as Linguist reads it.

#### Scenario: Shebang lines

- **WHEN** a caller reads `#!/usr/bin/env bash`, `#!/usr/bin/python3.11` and `#!/usr/bin/env -S deno run`
- **THEN** they give Shell, Python and JavaScript

### Requirement: A fence info string names its language by its first word

`by_fence` SHALL look up the first word of an info string as a name, and return the rest as its meta.

#### Scenario: A fence with a title

- **WHEN** a caller reads the info string `ts title="a.ts"`
- **THEN** it gives TypeScript and `title="a.ts"`

### Requirement: The committed table is the generated one

The table test SHALL fail when `src/table.rs` differs from what the vendored data and `data/local.yml` give, or when an entry of `data/local.yml` is not needed.

#### Scenario: A hand edit of the table

- **WHEN** a contributor edits `src/table.rs` by hand
- **THEN** the table test fails and names `mise run generate`
