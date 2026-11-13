//! Generates `src/table.rs` from the vendored Linguist data and
//! `data/local.yml`, and fails when the committed file differs.
//!
//! `mise run generate` writes the file; the checks of the table run in every
//! run.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Display};
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::Deserialize;
use serde::de::{DeserializeOwned, Deserializer, MapAccess, Visitor};

/// Why a check failed, printed as written so a multi-line message stays
/// readable.
struct Failure(String);

impl Debug for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<E: Display> From<E> for Failure {
    fn from(error: E) -> Self {
        Self(error.to_string())
    }
}

/// The values of Linguist's `type` field.
#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum KindName {
    Data,
    Markup,
    Programming,
    Prose,
}

/// The fields of a Linguist language that the table keeps.
#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "type")]
    kind: KindName,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    filenames: Vec<String>,
    #[serde(default)]
    interpreters: Vec<String>,
    fs_name: Option<String>,
}

/// Linguist's `languages.yml`: its languages in the order of the file.
struct Linguist(Vec<(String, Entry)>);

impl<'de> Deserialize<'de> for Linguist {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct InOrder;

        impl<'de> Visitor<'de> for InOrder {
            type Value = Linguist;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map of languages")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Linguist, A::Error> {
                let mut languages = Vec::new();
                while let Some(language) = map.next_entry::<String, Entry>()? {
                    languages.push(language);
                }
                Ok(Linguist(languages))
            }
        }

        deserializer.deserialize_map(InOrder)
    }
}

/// `data/local.yml`: what langdex sets on top of Linguist.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Local {
    first: BTreeMap<String, First>,
}

/// One extension whose first language `data/local.yml` sets.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct First {
    language: String,
    reason: String,
}

/// Everything the table is built from.
struct Data {
    languages: Vec<(String, Entry)>,
    popular: BTreeSet<String>,
    local: Local,
}

/// The indexes of the table, each sorted by its key bytes.
struct Indexes {
    names: BTreeMap<String, usize>,
    extensions: BTreeMap<String, Vec<usize>>,
    filenames: BTreeMap<String, Vec<usize>>,
    interpreters: BTreeMap<String, Vec<usize>>,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T, Failure> {
    let text = fs::read_to_string(path)
        .map_err(|error| Failure(format!("{}: {error}", path.display())))?;
    serde_saphyr::from_str(&text).map_err(|error| Failure(format!("{}: {error}", path.display())))
}

impl Data {
    fn read() -> Result<Self, Failure> {
        let root = root();
        let Linguist(languages) = read(&root.join("data/linguist/languages.yml"))?;
        let popular: Vec<String> = read(&root.join("data/linguist/popular.yml"))?;
        let local = read(&root.join("data/local.yml"))?;
        Ok(Self {
            languages,
            popular: popular.into_iter().collect(),
            local,
        })
    }

    fn is_popular(&self, index: usize) -> bool {
        self.popular.contains(&self.languages[index].0)
    }

    /// The names, `fs_name`s and aliases of each language, lowercased.
    fn names(&self) -> Result<BTreeMap<String, usize>, Failure> {
        let mut names: BTreeMap<String, usize> = BTreeMap::new();
        for (index, (name, entry)) in self.languages.iter().enumerate() {
            let keys = [name.as_str()]
                .into_iter()
                .chain(entry.fs_name.as_deref())
                .chain(entry.aliases.iter().map(String::as_str));
            for key in keys {
                let key = key.to_lowercase();
                if let Some(&other) = names.get(&key)
                    && other != index
                {
                    return Err(Failure(format!(
                        "the name `{key}` belongs to both {} and {}",
                        self.languages[other].0, name
                    )));
                }
                names.insert(key, index);
            }
        }
        Ok(names)
    }

    /// The languages of each extension, lowercased, in the order of the table.
    fn extensions(&self) -> Result<BTreeMap<String, Vec<usize>>, Failure> {
        let mut extensions: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, (name, entry)) in self.languages.iter().enumerate() {
            for extension in &entry.extensions {
                if !extension.starts_with('.') {
                    return Err(Failure(format!(
                        "{name} lists the extension `{extension}` with no leading dot"
                    )));
                }
                let languages = extensions.entry(extension.to_lowercase()).or_default();
                if !languages.contains(&index) {
                    languages.push(index);
                }
            }
        }
        for (extension, languages) in &mut extensions {
            languages.sort_by_cached_key(|&index| {
                let first = self.languages[index]
                    .1
                    .extensions
                    .first()
                    .is_some_and(|first| first.to_lowercase() == *extension);
                (
                    !self.is_popular(index),
                    !first,
                    self.languages[index].0.to_lowercase(),
                    index,
                )
            });
        }
        Ok(extensions)
    }

    /// The languages of each file name or interpreter, as written, in the order
    /// of the table.
    fn listed(&self, field: fn(&Entry) -> &[String]) -> BTreeMap<String, Vec<usize>> {
        let mut listed: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (index, (_, entry)) in self.languages.iter().enumerate() {
            for key in field(entry) {
                let languages = listed.entry(key.clone()).or_default();
                if !languages.contains(&index) {
                    languages.push(index);
                }
            }
        }
        for languages in listed.values_mut() {
            languages.sort_by_cached_key(|&index| {
                (
                    !self.is_popular(index),
                    self.languages[index].0.to_lowercase(),
                    index,
                )
            });
        }
        listed
    }

    /// Moves the language of each `first` entry to the front of its extension.
    fn apply_local(&self, extensions: &mut BTreeMap<String, Vec<usize>>) -> Result<(), Failure> {
        for (extension, first) in &self.local.first {
            if first.reason.trim().is_empty() {
                return Err(Failure(format!(
                    "data/local.yml: `{extension}` gives no reason"
                )));
            }
            let Some(languages) = extensions.get_mut(extension) else {
                return Err(Failure(format!(
                    "data/local.yml: no language lists the extension `{extension}`"
                )));
            };
            let Some(index) = self
                .languages
                .iter()
                .position(|(name, _)| *name == first.language)
            else {
                return Err(Failure(format!(
                    "data/local.yml: `{extension}` names the language {}, which Linguist does not list",
                    first.language
                )));
            };
            let Some(position) = languages.iter().position(|&listed| listed == index) else {
                return Err(Failure(format!(
                    "data/local.yml: {} does not list the extension `{extension}`",
                    first.language
                )));
            };
            if position == 0 {
                return Err(Failure(format!(
                    "data/local.yml: {} is already first for `{extension}`; remove the entry",
                    first.language
                )));
            }
            languages[..=position].rotate_right(1);
        }
        Ok(())
    }

    fn indexes(&self) -> Result<Indexes, Failure> {
        let mut extensions = self.extensions()?;
        self.apply_local(&mut extensions)?;
        Ok(Indexes {
            names: self.names()?,
            extensions,
            filenames: self.listed(|entry| &entry.filenames),
            interpreters: self.listed(|entry| &entry.interpreters),
        })
    }

    /// The exact text of `src/table.rs`.
    fn table(&self) -> Result<String, Failure> {
        let indexes = self.indexes()?;
        let languages: Vec<String> = self
            .languages
            .iter()
            .map(|(name, entry)| language(name, entry))
            .collect();
        let mut text = String::new();
        text.push_str(
            "// Generated by `cargo test --test table` with LANGDEX_GENERATE=1, which is `mise run generate`.\n\
             // Do not edit by hand: the table test fails when this file differs from what it writes.\n\
             // Source: GitHub Linguist at the commit in data/linguist/COMMIT, and data/local.yml.\n\
             \n\
             use crate::{Kind, Language};\n\
             \n",
        );
        text.push_str(&section(
            "Every language, in Linguist's order.",
            "LANGUAGES",
            &format!("[Language; {}]", languages.len()),
            &languages,
        ));
        let names: Vec<String> = indexes
            .names
            .iter()
            .map(|(key, index)| format!("({key:?}, {index})"))
            .collect();
        text.push_str(&section(
            "Every name, file system name and alias, lowercased, to its language.",
            "NAMES",
            &format!("[(&str, usize); {}]", names.len()),
            &names,
        ));
        for (title, constant, map) in [
            (
                "Every extension, lowercased, to its languages in order.",
                "EXTENSIONS",
                &indexes.extensions,
            ),
            (
                "Every file name, as written, to its languages in order.",
                "FILENAMES",
                &indexes.filenames,
            ),
            (
                "Every interpreter to its languages in order.",
                "INTERPRETERS",
                &indexes.interpreters,
            ),
        ] {
            let rows: Vec<String> = map
                .iter()
                .map(|(key, languages)| format!("({key:?}, &{languages:?})"))
                .collect();
            text.push_str(&section(
                title,
                constant,
                &format!("[(&str, &[usize]); {}]", rows.len()),
                &rows,
            ));
        }
        Ok(format!("{}\n", text.trim_end()))
    }
}

/// One generated static, one row per line.
fn section(doc: &str, constant: &str, ty: &str, rows: &[String]) -> String {
    let body: String = rows
        .iter()
        .map(|row| ["    ", row, ",\n"].concat())
        .collect();
    format!("/// {doc}\npub(crate) static {constant}: {ty} = [\n{body}];\n\n")
}

/// One row of `LANGUAGES`.
fn language(name: &str, entry: &Entry) -> String {
    let kind = match entry.kind {
        KindName::Data => "Data",
        KindName::Markup => "Markup",
        KindName::Programming => "Programming",
        KindName::Prose => "Prose",
    };
    format!(
        "Language {{ name: {name:?}, kind: Kind::{kind}, aliases: {}, extensions: {}, filenames: {}, interpreters: {} }}",
        list(&entry.aliases),
        list(&entry.extensions),
        list(&entry.filenames),
        list(&entry.interpreters),
    )
}

fn list(items: &[String]) -> String {
    let items: Vec<String> = items.iter().map(|item| format!("{item:?}")).collect();
    format!("&[{}]", items.join(", "))
}

#[test]
fn names_are_unique_after_lowercasing() -> Result<(), Failure> {
    Data::read()?.names().map(drop)
}

#[test]
fn extensions_start_with_a_dot() -> Result<(), Failure> {
    Data::read()?.extensions().map(drop)
}

#[test]
fn popular_languages_exist() -> Result<(), Failure> {
    let data = Data::read()?;
    let known: BTreeSet<&str> = data
        .languages
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    match data
        .popular
        .iter()
        .find(|name| !known.contains(name.as_str()))
    {
        Some(name) => Err(Failure(format!(
            "popular.yml lists {name}, which languages.yml does not"
        ))),
        None => Ok(()),
    }
}

#[test]
fn local_entries_are_needed() -> Result<(), Failure> {
    let data = Data::read()?;
    let mut extensions = data.extensions()?;
    data.apply_local(&mut extensions)
}

#[test]
fn table_is_the_generated_one() -> Result<(), Failure> {
    let table = Data::read()?.table()?;
    let path = root().join("src/table.rs");
    if env::var("LANGDEX_GENERATE").is_ok_and(|value| value == "1") {
        return Ok(fs::write(&path, table)?);
    }
    let committed = fs::read_to_string(&path)?;
    if committed == table {
        return Ok(());
    }
    let line = committed
        .lines()
        .zip(table.lines())
        .position(|(old, new)| old != new)
        .unwrap_or_else(|| committed.lines().count().min(table.lines().count()));
    Err(Failure(format!(
        "src/table.rs differs from the generated table at line {}.\n\
         Run `mise run generate` and commit the result.",
        line + 1
    )))
}
