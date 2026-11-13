# langdex

The languages of source files as one table: each language's name, aliases, file extensions, file names and interpreters.  
langdex answers which language a file or a block of code holds, by any name the language goes by.  
It is a Rust library, not a command.

## What it answers

The first release is planned to answer these lookups:

- `md`, `markdown` and `Markdown` name Markdown, since a name matches without regard to case.
- A file `app.mjs` is JavaScript, and a file `app.mts` is TypeScript.
- A file named `Makefile` is Makefile.
- A file whose first line is `#!/usr/bin/env bash` is Shell.
- The fence info string `ts title="a.ts"` names TypeScript, and `title="a.ts"` stays as its meta.

## Data

The table is generated from GitHub Linguist's `languages.yml` at a pinned commit, with a small file of local entries beside it.  
The generated table is committed, so a build reads nothing from the network.

## License

MIT OR Apache-2.0.  
The Linguist data is MIT, and its notice ships with the generated table.
