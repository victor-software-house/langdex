//! The languages of source files and blocks of code, as one table.
//!
//! Each language has a name, aliases, file extensions, file names and
//! interpreters, generated from GitHub Linguist.
//! The crate answers which language a name, a path, a first line or a fence
//! holds, and reads no file and starts no process.
//!
//! - `md`, `markdown` and `Markdown` name Markdown, since a name matches
//!   without regard to case.
//! - A file `app.mjs` is JavaScript, and a file `app.mts` is TypeScript.
//! - A file named `Makefile` is Makefile.
//! - A file whose first line is `#!/usr/bin/env bash` is Shell.
//! - The fence info string `ts title="a.ts"` names TypeScript, and
//!   `title="a.ts"` stays as its meta.
//!
//! ```
//! use langdex::{by_fence, by_first_line, by_name, by_path};
//!
//! assert_eq!(
//!     by_name("md").map(|language| language.name),
//!     Some("Markdown")
//! );
//! assert_eq!(
//!     by_path("src/app.mts").map(|language| language.name),
//!     Some("TypeScript")
//! );
//! assert_eq!(
//!     by_path("Makefile").map(|language| language.name),
//!     Some("Makefile")
//! );
//! assert_eq!(
//!     by_first_line("#!/usr/bin/env bash").map(|language| language.name),
//!     Some("Shell")
//! );
//!
//! let (language, meta) = by_fence("ts title=\"a.ts\"").expect("a known fence");
//! assert_eq!(language.name, "TypeScript");
//! assert_eq!(meta, "title=\"a.ts\"");
//! ```
//!
//! An extension that several languages list gives them in one order: the
//! languages Linguist calls popular, then the languages whose first extension
//! it is, then the name.
//! A lookup that returns one language returns the first, and the `all_by_*`
//! lookups return every one in that order.

#[rustfmt::skip]
mod table;

use table::{EXTENSIONS, FILENAMES, INTERPRETERS, LANGUAGES, NAMES};

/// The kind of a language, as Linguist classifies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kind {
    /// A language of data, such as JSON or YAML.
    Data,
    /// A markup language, such as HTML or Markdown.
    Markup,
    /// A programming language.
    Programming,
    /// A language of prose, such as plain text.
    Prose,
}

/// One language of the table.
///
/// The fields are Linguist's, in Linguist's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    /// The name, such as `Markdown`.
    pub name: &'static str,
    /// What Linguist classifies the language as.
    pub kind: Kind,
    /// Other names of the language, such as `md` and `pandoc` for Markdown.
    pub aliases: &'static [&'static str],
    /// The file extensions, each with its leading dot.
    /// The first is the language's primary extension.
    pub extensions: &'static [&'static str],
    /// The file names that hold the language, such as `Makefile`.
    pub filenames: &'static [&'static str],
    /// The programs that run the language, as a shebang line names them.
    pub interpreters: &'static [&'static str],
}

/// Every language of the table, in Linguist's order.
///
/// # Examples
///
/// ```
/// assert!(
///     langdex::all()
///         .iter()
///         .any(|language| language.name == "Rust")
/// );
/// ```
#[must_use]
pub fn all() -> &'static [Language] {
    &LANGUAGES
}

/// Finds a language by its name, its file system name or one of its aliases,
/// without regard to case.
///
/// A name that matches none is read as an extension, as [`by_extension`] reads
/// it, so `mts` names TypeScript.
///
/// # Examples
///
/// ```
/// use langdex::by_name;
///
/// for name in ["md", "markdown", "Markdown"] {
///     assert_eq!(
///         by_name(name).map(|language| language.name),
///         Some("Markdown")
///     );
/// }
/// assert_eq!(
///     by_name("mts").map(|language| language.name),
///     Some("TypeScript")
/// );
/// assert_eq!(by_name("no such language"), None);
/// ```
#[must_use]
pub fn by_name(name: &str) -> Option<&'static Language> {
    find(&NAMES, &name.to_lowercase())
        .map(|index| &LANGUAGES[index])
        .or_else(|| by_extension(name))
}

/// Finds the first language of an extension, written `md` or `.md`, without
/// regard to case.
///
/// # Examples
///
/// ```
/// use langdex::by_extension;
///
/// assert_eq!(
///     by_extension("rs").map(|language| language.name),
///     Some("Rust")
/// );
/// assert_eq!(
///     by_extension(".RS").map(|language| language.name),
///     Some("Rust")
/// );
/// ```
#[must_use]
pub fn by_extension(ext: &str) -> Option<&'static Language> {
    all_by_extension(ext).next()
}

/// Lists every language of an extension, written `md` or `.md`, without regard
/// to case, in the order the crate documents.
///
/// # Examples
///
/// ```
/// use langdex::all_by_extension;
///
/// let names: Vec<_> = all_by_extension("h")
///     .map(|language| language.name)
///     .collect();
/// assert_eq!(names, ["C", "C++", "Objective-C"]);
/// ```
pub fn all_by_extension(ext: &str) -> impl Iterator<Item = &'static Language> + use<> {
    let lower = ext.to_lowercase();
    let key = if lower.starts_with('.') {
        lower
    } else {
        format!(".{lower}")
    };
    languages(find(&EXTENSIONS, &key).unwrap_or_default())
}

/// Finds the first language of a file name, which must match exactly.
///
/// # Examples
///
/// ```
/// use langdex::by_filename;
///
/// assert_eq!(
///     by_filename("Makefile").map(|language| language.name),
///     Some("Makefile")
/// );
/// assert_eq!(by_filename("MAKEFILE"), None);
/// ```
#[must_use]
pub fn by_filename(name: &str) -> Option<&'static Language> {
    find(&FILENAMES, name)
        .and_then(<[usize]>::first)
        .map(|&index| &LANGUAGES[index])
}

/// Finds the first language of a path.
///
/// The file name is the part after the last `/`.
/// A language that lists the file name wins, and then a language that lists an
/// extension of the file name, taking the longest extension first.
/// So `a.d.ts` tries `.d.ts` and then `.ts`.
///
/// # Examples
///
/// ```
/// use langdex::by_path;
///
/// assert_eq!(
///     by_path("docs/a.md").map(|language| language.name),
///     Some("Markdown")
/// );
/// assert_eq!(
///     by_path("a.d.ts").map(|language| language.name),
///     Some("TypeScript")
/// );
/// assert_eq!(
///     by_path("home/.bashrc").map(|language| language.name),
///     Some("Shell")
/// );
/// ```
#[must_use]
pub fn by_path(path: &str) -> Option<&'static Language> {
    all_by_path(path).next()
}

/// Lists every language of the first key that matches a path, in the order
/// the crate documents.
///
/// The keys are those of [`by_path`]: the file name, then each extension.
///
/// # Examples
///
/// ```
/// use langdex::all_by_path;
///
/// let names: Vec<_> = all_by_path("src/a.h")
///     .map(|language| language.name)
///     .collect();
/// assert_eq!(names, ["C", "C++", "Objective-C"]);
/// ```
pub fn all_by_path(path: &str) -> impl Iterator<Item = &'static Language> + use<> {
    let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    let indexes = find(&FILENAMES, name).or_else(|| {
        let lower = name.to_lowercase();
        lower
            .match_indices('.')
            .find_map(|(start, _)| find(&EXTENSIONS, &lower[start..]))
    });
    languages(indexes.unwrap_or_default())
}

/// Finds the first language of an interpreter, which must match exactly.
///
/// # Examples
///
/// ```
/// use langdex::by_interpreter;
///
/// assert_eq!(
///     by_interpreter("node").map(|language| language.name),
///     Some("JavaScript")
/// );
/// assert_eq!(
///     by_interpreter("python3").map(|language| language.name),
///     Some("Python")
/// );
/// ```
#[must_use]
pub fn by_interpreter(name: &str) -> Option<&'static Language> {
    find(&INTERPRETERS, name)
        .and_then(<[usize]>::first)
        .map(|&index| &LANGUAGES[index])
}

/// Reads the interpreter of a shebang line as Linguist's `Shebang.interpreter`
/// does.
///
/// The line starts with `#!`, and the path is the first word after it.
/// The script is the part after the last `/` of the path.
/// When the script is `env`, the options (`-` with any of `i0uCSv`, or `--` and
/// a word) and the `NAME=value` arguments after it are skipped, and the script
/// is the next word.
/// A trailing `.` and digits are dropped, so `python3.11` reads `python3`.
/// An `osascript` with a later `-l` gives no interpreter.
///
/// Linguist also reads an `exec` line in the first five lines of an `sh`
/// script. This function reads one line, so it does not.
///
/// # Examples
///
/// ```
/// use langdex::interpreter;
///
/// assert_eq!(interpreter("#!/usr/bin/env bash"), Some("bash"));
/// assert_eq!(interpreter("#!/usr/bin/env -S deno run"), Some("deno"));
/// assert_eq!(interpreter("#!/usr/bin/python3.11"), Some("python3"));
/// assert_eq!(interpreter("#!/usr/bin/osascript -l JavaScript"), None);
/// assert_eq!(interpreter("print('no shebang')"), None);
/// ```
#[must_use]
pub fn interpreter(line: &str) -> Option<&str> {
    let mut words = line.lines().next()?.strip_prefix("#!")?.split_whitespace();
    let mut script = words.next()?.rsplit('/').next()?;
    if script == "env" {
        script = words.find(|word| !is_env_option(word) && !is_assignment(word))?;
    }
    let script = strip_version(script);
    if script == "osascript" && words.any(|word| word == "-l") {
        return None;
    }
    (!script.is_empty()).then_some(script)
}

/// Finds the first language of the interpreter that a first line names.
///
/// # Examples
///
/// ```
/// use langdex::by_first_line;
///
/// let name = |line| by_first_line(line).map(|language| language.name);
/// assert_eq!(name("#!/usr/bin/env bash"), Some("Shell"));
/// assert_eq!(name("#!/usr/bin/python3.11"), Some("Python"));
/// assert_eq!(name("#!/usr/bin/env -S deno run"), Some("JavaScript"));
/// ```
#[must_use]
pub fn by_first_line(line: &str) -> Option<&'static Language> {
    interpreter(line).and_then(by_interpreter)
}

/// Reads the info string of a fenced code block.
///
/// The first word, up to the first whitespace, names the language as
/// [`by_name`] reads a name. The rest of the string, trimmed, is its meta. An
/// empty word gives `None`.
///
/// # Examples
///
/// ```
/// use langdex::by_fence;
///
/// let (language, meta) = by_fence("ts title=\"a.ts\"").expect("a known fence");
/// assert_eq!(language.name, "TypeScript");
/// assert_eq!(meta, "title=\"a.ts\"");
///
/// assert_eq!(
///     by_fence("md").map(|(language, _)| language.name),
///     Some("Markdown")
/// );
/// assert_eq!(by_fence(""), None);
/// ```
#[must_use]
pub fn by_fence(info: &str) -> Option<(&'static Language, &str)> {
    let info = info.trim();
    let (word, meta) = info
        .split_once(char::is_whitespace)
        .map_or((info, ""), |(word, meta)| (word, meta.trim_start()));
    if word.is_empty() {
        return None;
    }
    by_name(word).map(|language| (language, meta))
}

/// Looks a key up in an index sorted by its key bytes.
fn find<T: Copy>(index: &'static [(&'static str, T)], key: &str) -> Option<T> {
    index
        .binary_search_by_key(&key, |&(name, _)| name)
        .ok()
        .map(|position| index[position].1)
}

/// The languages at some positions of the table.
fn languages(indexes: &'static [usize]) -> impl Iterator<Item = &'static Language> + use<> {
    indexes.iter().map(|&index| &LANGUAGES[index])
}

/// Whether an `env` argument is an option, such as `-S` or
/// `--ignore-environment`.
fn is_env_option(word: &str) -> bool {
    word.starts_with("--")
        || word
            .strip_prefix('-')
            .is_some_and(|flags| flags.chars().all(|flag| "i0uCSv".contains(flag)))
}

/// Whether an `env` argument is a `NAME=value` assignment.
fn is_assignment(word: &str) -> bool {
    word.split_once('=')
        .is_some_and(|(name, _)| !name.is_empty())
}

/// Drops a trailing `.` and digits, so `python3.11` reads `python3`.
fn strip_version(script: &str) -> &str {
    script
        .rsplit_once('.')
        .filter(|(_, version)| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map_or(script, |(name, _)| name)
}
