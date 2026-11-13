# Design

## 1. Data

`data/linguist/` holds `languages.yml` and `popular.yml` from Linguist tag `v9.7.0` (commit `e0c78d62c42abae6122235d8e68a7aa43eef89da`), byte for byte, and `SHA256SUMS` with their hashes.  
`LICENSE-LINGUIST` at the root holds Linguist's MIT license, so `cargo package` packs it beside the table derived from that data.  
The mise task `linguist:fetch` downloads both files at the commit in `data/linguist/COMMIT` and checks them with `shasum -a 256 -c SHA256SUMS`.  
Moving to a newer Linguist changes `COMMIT`, runs the fetch, rewrites `SHA256SUMS`, and regenerates the table, in one pull request.

`data/local.yml` holds what langdex sets on top of Linguist, each entry with a `reason`:

```yaml
first:
  .md: {language: Markdown, reason: GCC Machine Description is a niche language that sorts first by name.}
```

The first release sets the first language of seven extensions: `.md` Markdown, `.yaml` YAML, `.txt` Text, `.cfg` INI, `.asc` AsciiDoc, `.v` Verilog and `.frag` GLSL.  
Without them, `.md` gives GCC Machine Description, `.yaml` MiniYAML, `.txt` Adblock Filter List, `.cfg` HAProxy, `.asc` AGS Script, `.v` Rocq Prover, and `.frag` JavaScript.  
`.sql` needs no entry: the order puts SQL before SQLPL by name.

## 2. Order

Each shared extension, file name and interpreter lists its languages in one order, and a lookup that returns one language returns the first:

1. a language that `popular.yml` lists;
2. for an extension, a language whose first Linguist extension it is;
3. the language name, compared without regard to case.

Then `first` in `data/local.yml` moves its language to the front of that extension's list.  
The generator test fails when an entry names a language that does not list the extension, or one that the order already puts first, so every entry stays needed.

| Lookup | First | Why |
|:--|:--|:--|
| `a.h` | C | C, C++ and Objective-C are all popular, and C is first by name |
| `a.rs` | Rust | popular |
| `a.md` | Markdown | `data/local.yml` |
| `a.cl` | Cool | no language is popular, all have it first, and Cool is first by name |
| interpreter `node` | JavaScript | both JavaScript and TypeScript are popular |

## 3. Table

`src/table.rs` is generated in one fixed layout, one entry per line, and starts with a comment that names the test that writes it.  
`lib.rs` declares it as `#[rustfmt::skip] mod table;`, so rustfmt leaves the layout as the generator wrote it and the drift check compares plain text.  
It holds `LANGUAGES`, a static array of `Language` in Linguist's order, and four indexes, each a static array sorted by its key bytes:

- `NAMES: [(&str, usize); _]`: every name, `fs_name` and alias, lowercased with `str::to_lowercase`, to its language;
- `EXTENSIONS: [(&str, &[usize]); _]`: every extension, lowercased, with its leading dot, to its languages in order;
- `FILENAMES: [(&str, &[usize]); _]`: every file name, as written, to its languages in order;
- `INTERPRETERS: [(&str, &[usize]); _]`: every interpreter to its languages in order.

A lookup is a binary search on one index, with no allocation except the lowercased key.

## 4. Types

```rust
pub enum Kind { Data, Markup, Programming, Prose }

pub struct Language {
    pub name: &'static str,
    pub kind: Kind,
    pub aliases: &'static [&'static str],
    pub extensions: &'static [&'static str],
    pub filenames: &'static [&'static str],
    pub interpreters: &'static [&'static str],
}
```

Fields are Linguist's, in Linguist's order: `extensions[0]` is the language's first extension, with its dot.  
Both types derive `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq`; `Kind` is `#[non_exhaustive]`.

## 5. Lookups

| Function | Reads | Returns |
|:--|:--|:--|
| `all()` | the table | `&'static [Language]` |
| `by_name(name)` | a name, `fs_name` or alias, without regard to case; else `by_extension(name)` | `Option<&'static Language>` |
| `by_extension(ext)` | `md` or `.md`, without regard to case | the first language |
| `all_by_extension(ext)` | the same | every language, in order |
| `by_filename(name)` | a file name, exactly | the first language |
| `by_path(path)` | the part after the last `/`: its file name, then its extensions | the first language |
| `all_by_path(path)` | the same | every language of the first key that matches |
| `by_interpreter(name)` | an interpreter, exactly | the first language |
| `interpreter(line)` | the first line of a file | the interpreter, as Linguist reads a shebang |
| `by_first_line(line)` | `interpreter(line)`, then `by_interpreter` | the first language |
| `by_fence(info)` | the first word of a fence info string, by `by_name` | the language and the rest of the string, trimmed |

Every `all_*` returns `impl Iterator<Item = &'static Language>`.

`by_path` reads the extensions of a file name as Linguist's `FileBlob#extensions` does: the name lowercased and split at each dot, the part before the first dot dropped, then each suffix from the longest.  
So `a.d.ts` tries `.d.ts`, then `.ts`, and `.bashrc` tries `.bashrc`.  
The first key with a language wins, file name before extensions.

`interpreter` follows Linguist's `Shebang.interpreter` (`lib/linguist/shebang.rb` at the pinned tag) on one line:

1. The line starts with `#!`; the path is the first run of non-space characters after it and any spaces.
2. The script is the part after the path's last `/`.
3. When the script is `env`, skip spaces, then any run of options and of `NAME=value` arguments, each with the spaces after it; the script is the next word.  
   An option is `-` with any of `i0uCSv`, or `--` and at least one more character, and an argument has a name and a value.
4. A trailing `.` and digits are dropped, so `python3.11` reads `python3`.
5. `osascript` with `-l` anywhere later in the line, as in `-lJavaScript`, gives no interpreter.

Linguist also reads an `exec` line in the first five lines of an `sh` script; `interpreter` reads one line, so it does not, which the rustdoc of `interpreter` records.

`by_fence` takes the info string of a fenced code block: the word is the text up to the first whitespace, and an empty word gives `None`.  
So `ts title="a.ts"` gives TypeScript and `title="a.ts"`, and `md` and `markdown` both give Markdown.

## 6. Generator and checks

`tests/table.rs` reads `data/linguist/*.yml` and `data/local.yml` through `vsh-serde-saphyr` into typed structs that refuse unknown fields of `data/local.yml`, and builds the text of `src/table.rs`.  
With `LANGDEX_GENERATE=1` it writes the file; without, it fails when the committed file differs, and prints the command that regenerates it.  
The mise task `generate` runs `LANGDEX_GENERATE=1 cargo test --test table`.  
It also checks what the generator assumes: names, `fs_name`s and aliases are unique after lowercasing, every extension starts with a dot, and every entry of `data/local.yml` is needed.

`Cargo.toml` packs `src/`, `README.md`, `LICENSE-*` and `Cargo.toml` only, so the vendored data and the tests stay out of the crate.

## 7. Size

The table holds 833 languages, 1,486 extensions, 419 file names and 168 interpreters.  
The tests record the size of a release build of a small binary that links langdex, against one that does not, in the pull request.
