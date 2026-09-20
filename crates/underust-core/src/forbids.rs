//! The CONSTRAIN ban, enforced against a parsed AST.
//!
//! A ban is data, declared as `forbids` in `task.toml`, so a CI gate can prove each one
//! actually fires. Matching happens on method names, path segments and macro names --
//! never on raw text, or a comment mentioning `clone` would fail an honest solution.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use syn::spanned::Spanned as _;
use syn::visit::Visit;

/// One use of a forbidden construct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The banned name that matched.
    pub construct: String,
    /// One-based line within the file.
    pub line: usize,
}

/// Everything that can go wrong checking a ban.
#[derive(Debug, thiserror::Error)]
pub enum ForbidsError {
    /// The source could not be parsed as Rust.
    #[error("parsing {path}: {source}")]
    Parse {
        /// The file involved, or `<memory>` for a string.
        path: String,
        /// The underlying failure.
        source: syn::Error,
    },
    /// A file could not be read.
    #[error("reading {path}: {source}")]
    Io {
        /// The file involved.
        path: PathBuf,
        /// The underlying failure.
        source: std::io::Error,
    },
}

struct Hunt<'a> {
    banned: &'a BTreeSet<&'a str>,
    found: Vec<Violation>,
}

impl Hunt<'_> {
    fn note(&mut self, name: &str, line: usize) {
        if self.banned.contains(name) {
            self.found.push(Violation {
                construct: name.to_owned(),
                line,
            });
        }
    }
}

impl<'ast> Visit<'ast> for Hunt<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.note(&node.method.to_string(), node.method.span().start().line);
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_path_segment(&mut self, node: &'ast syn::PathSegment) {
        self.note(&node.ident.to_string(), node.ident.span().start().line);
        syn::visit::visit_path_segment(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(last) = node.path.segments.last() {
            self.note(&last.ident.to_string(), last.ident.span().start().line);
        }
        // Deliberately does not recurse into the macro's token stream: its contents are
        // not parseable as expressions in general.
    }
}

/// Check one Rust source string against a ban list.
///
/// # Errors
/// Returns [`ForbidsError::Parse`] when the source is not valid Rust. A ban that cannot be
/// checked must fail loudly rather than pass silently.
pub fn check_source(src: &str, forbids: &[String]) -> Result<Vec<Violation>, ForbidsError> {
    let banned: BTreeSet<&str> = forbids.iter().map(String::as_str).collect();
    if banned.is_empty() {
        return Ok(Vec::new());
    }
    let file = syn::parse_file(src).map_err(|source| ForbidsError::Parse {
        path: "<memory>".to_owned(),
        source,
    })?;
    let mut hunt = Hunt {
        banned: &banned,
        found: Vec::new(),
    };
    hunt.visit_file(&file);
    Ok(hunt.found)
}

/// Check every `.rs` file beneath `root`.
///
/// # Errors
/// Propagates read and parse failures.
pub fn check_dir(root: &Path, forbids: &[String]) -> Result<Vec<Violation>, ForbidsError> {
    let mut all = Vec::new();
    let mut stack = vec![root.to_owned()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|source| ForbidsError::Io {
            path: dir.clone(),
            source,
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let src = std::fs::read_to_string(&path).map_err(|source| ForbidsError::Io {
                    path: path.clone(),
                    source,
                })?;
                all.extend(check_source(&src, forbids)?);
            }
        }
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bans() -> Vec<String> {
        ["clone", "to_owned", "Rc", "RefCell"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect()
    }

    #[test]
    fn flags_a_method_call() {
        let src = "fn f(v: &Vec<u8>) -> Vec<u8> { v.clone() }";
        let found = check_source(src, &bans()).expect("parse");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].construct, "clone");
        assert_eq!(found[0].line, 1);
    }

    #[test]
    fn flags_a_type_path() {
        let src = "use std::rc::Rc; fn f() -> Rc<u8> { Rc::new(1) }";
        let found = check_source(src, &bans()).expect("parse");
        assert!(found.iter().any(|v| v.construct == "Rc"));
    }

    #[test]
    fn does_not_flag_a_comment() {
        let src = "// deliberately avoids clone and Rc here\nfn f(v: &[u8]) -> usize { v.len() }";
        assert!(check_source(src, &bans()).expect("parse").is_empty());
    }

    #[test]
    fn does_not_flag_a_string_literal() {
        let src = r#"fn f() -> &'static str { "clone" }"#;
        assert!(check_source(src, &bans()).expect("parse").is_empty());
    }

    #[test]
    fn does_not_flag_an_identifier_that_merely_contains_the_word() {
        let src = "fn f() -> bool { let cloned_flag = true; cloned_flag }";
        assert!(check_source(src, &bans()).expect("parse").is_empty());
    }

    #[test]
    fn an_empty_ban_list_flags_nothing() {
        let src = "fn f(v: &Vec<u8>) -> Vec<u8> { v.clone() }";
        assert!(check_source(src, &[]).expect("parse").is_empty());
    }

    #[test]
    fn unparseable_source_is_an_error_not_a_silent_pass() {
        assert!(check_source("fn f( {", &bans()).is_err());
    }

    #[test]
    fn reports_the_line_a_violation_is_on() {
        let src = "fn a() {}\nfn b() {}\nfn c(v: &Vec<u8>) -> Vec<u8> { v.clone() }";
        let found = check_source(src, &bans()).expect("parse");
        assert_eq!(found[0].line, 3, "a solver needs to be told where");
    }
}
