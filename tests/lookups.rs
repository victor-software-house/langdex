//! The lookups of the `language-table` spec and the order of a shared
//! extension.

use langdex::{
    Language, all_by_extension, all_by_path, by_extension, by_fence, by_first_line, by_interpreter,
    by_name, by_path, interpreter,
};

fn name(language: Option<&Language>) -> Option<&str> {
    language.map(|language| language.name)
}

fn names(languages: impl Iterator<Item = &'static Language>) -> Vec<&'static str> {
    languages.map(|language| language.name).collect()
}

#[test]
fn names_of_markdown() {
    for given in ["md", "markdown", "Markdown"] {
        assert_eq!(name(by_name(given)), Some("Markdown"), "{given}");
    }
}

#[test]
fn an_extension_as_a_name() {
    assert_eq!(name(by_name("mts")), Some("TypeScript"));
}

#[test]
fn a_header_file() {
    assert_eq!(names(all_by_extension("h")), ["C", "C++", "Objective-C"]);
    assert_eq!(names(all_by_path("src/a.h")), ["C", "C++", "Objective-C"]);
}

#[test]
fn a_markdown_file() {
    assert_eq!(name(by_path("docs/a.md")), Some("Markdown"));
}

#[test]
fn files_by_name_and_extension() {
    assert_eq!(name(by_path("Makefile")), Some("Makefile"));
    assert_eq!(name(by_path("app.mjs")), Some("JavaScript"));
    assert_eq!(name(by_path("app.mts")), Some("TypeScript"));
}

#[test]
fn a_path_reads_its_longest_extension_first() {
    assert_eq!(name(by_path("a.d.ts")), Some("TypeScript"));
}

#[test]
fn a_dotfile_is_a_file_name() {
    assert_eq!(name(by_path(".bashrc")), Some("Shell"));
    assert_eq!(name(by_path("home/.bashrc")), Some("Shell"));
}

#[test]
fn shebang_lines() {
    assert_eq!(name(by_first_line("#!/usr/bin/env bash")), Some("Shell"));
    assert_eq!(name(by_first_line("#!/usr/bin/python3.11")), Some("Python"));
    assert_eq!(
        name(by_first_line("#!/usr/bin/env -S deno run")),
        Some("JavaScript")
    );
}

#[test]
fn an_env_line_skips_options_and_assignments() {
    assert_eq!(interpreter("#!/usr/bin/env -S deno run"), Some("deno"));
    assert_eq!(
        interpreter("#!/usr/bin/env LC_ALL=C python3"),
        Some("python3")
    );
    assert_eq!(
        interpreter("#!/usr/bin/env -i LC_ALL=C perl -w"),
        Some("perl")
    );
}

#[test]
fn a_shebang_with_a_space_and_arguments() {
    assert_eq!(interpreter("#! /usr/bin/perl -w"), Some("perl"));
    assert_eq!(name(by_first_line("#! /usr/bin/perl -w")), Some("Perl"));
}

#[test]
fn a_version_is_dropped() {
    assert_eq!(interpreter("#!/usr/bin/python3.11"), Some("python3"));
}

#[test]
fn a_bare_double_dash_or_an_empty_value_ends_the_env_arguments() {
    assert_eq!(interpreter("#!/usr/bin/env -- python"), Some("--"));
    assert_eq!(interpreter("#!/usr/bin/env FOO= python"), Some("FOO="));
}

#[test]
fn osascript_with_a_language_has_no_interpreter() {
    assert_eq!(interpreter("#!/usr/bin/osascript -l JavaScript"), None);
    assert_eq!(interpreter("#!/usr/bin/osascript -lJavaScript"), None);
    assert_eq!(
        name(by_first_line("#!/usr/bin/osascript -l JavaScript")),
        None
    );
    assert_eq!(interpreter("#!/usr/bin/osascript"), Some("osascript"));
}

#[test]
fn a_line_without_a_shebang_has_no_interpreter() {
    assert_eq!(interpreter("print('hello')"), None);
    assert_eq!(interpreter("#!"), None);
}

#[test]
fn a_fence_with_a_title() {
    let (language, meta) = by_fence("ts title=\"a.ts\"").expect("TypeScript is a known fence");
    assert_eq!(language.name, "TypeScript");
    assert_eq!(meta, "title=\"a.ts\"");
}

#[test]
fn a_fence_without_meta_or_a_word() {
    assert_eq!(
        by_fence("md").map(|(language, meta)| (language.name, meta)),
        Some(("Markdown", ""))
    );
    assert_eq!(by_fence(""), None);
    assert_eq!(by_fence("   "), None);
}

#[test]
fn the_first_language_of_a_shared_extension() {
    assert_eq!(name(by_path("a.h")), Some("C"));
    assert_eq!(name(by_path("a.rs")), Some("Rust"));
    assert_eq!(name(by_path("a.md")), Some("Markdown"));
    assert_eq!(name(by_path("a.cl")), Some("Cool"));
    assert_eq!(name(by_interpreter("node")), Some("JavaScript"));
    assert_eq!(name(by_extension(".RS")), Some("Rust"));
}
