//! Pure source analysis for the repository's explicit, single-assertion tests.

use proc_macro2::{Span, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{Attribute, Block, ItemFn, Macro, Meta, parse::Parser, punctuated::Punctuated};

#[derive(Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl Diagnostic {
    fn new(span: Span, message: String) -> Self {
        Self {
            line: span.start().line,
            column: span.start().column + 1,
            message,
        }
    }
}

#[derive(Debug, Default)]
pub struct Report {
    pub tests: usize,
    pub diagnostics: Vec<Diagnostic>,
}

/// Analyze all configurations, including disabled and ignored tests, without expansion.
pub fn analyze(source: &str) -> Result<Report, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut checker = Checker::default();
    checker.visit_file(&syntax);
    Ok(checker.report)
}

#[derive(Default)]
struct Checker {
    report: Report,
}

impl Checker {
    fn function(&mut self, attrs: &[Attribute], name: &syn::Ident, body: &Block) {
        let test = attrs.iter().any(|attr| test_meta(&attr.meta));
        let body = inspect_body(body);
        self.report.tests += usize::from(test);
        self.report.diagnostics.extend(body.diagnostics);
        self.report.diagnostics.extend(attribute_diagnostics(attrs));
        if let Some(message) = assertion_error(&name.to_string(), test, body.assertions) {
            self.report
                .diagnostics
                .push(Diagnostic::new(name.span(), message));
        }
    }
}

impl<'ast> Visit<'ast> for Checker {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.function(&node.attrs, &node.sig.ident, &node.block);
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.function(&node.attrs, &node.sig.ident, &node.block);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if let Some(body) = &node.default {
            self.function(&node.attrs, &node.sig.ident, body);
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        // Fuzz targets are separate executables, not generated unit tests.
        if node
            .mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "fuzz_target")
        {
            let audit = inspect_tokens(node.mac.tokens.clone());
            self.report.diagnostics.extend(audit.diagnostics);
            if let Some(message) = assertion_error("fuzz_target", true, audit.assertions) {
                self.report
                    .diagnostics
                    .push(Diagnostic::new(node.mac.bang_token.span, message));
            }
        } else {
            self.report.diagnostics.push(Diagnostic::new(node.mac.bang_token.span,
                "item macros and generated tests require explicit checker support; use explicit #[test] functions".into()));
        }
    }
}

fn assertion_error(name: &str, test: bool, assertions: usize) -> Option<String> {
    match (test, assertions) {
        (true, 1) | (false, 0) => None,
        (true, count) => Some(format!(
            "test {name}: expected exactly one assertion, found {count}"
        )),
        (false, count) => Some(format!(
            "helper {name}: assertions belong in test functions, found {count}"
        )),
    }
}

fn test_meta(meta: &Meta) -> bool {
    if meta
        .path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test")
    {
        return true;
    }
    nested_attributes(meta).iter().any(test_meta)
}

fn nested_attributes(meta: &Meta) -> Vec<Meta> {
    match meta {
        Meta::List(list) if list.path.is_ident("cfg_attr") => {
            Punctuated::<Meta, syn::token::Comma>::parse_terminated
                .parse2(list.tokens.clone())
                .map(|items| items.into_iter().skip(1).collect())
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

fn supported_attribute(meta: &Meta) -> bool {
    let Some(segment) = meta.path().segments.last() else {
        return false;
    };
    match segment.ident.to_string().as_str() {
        "cfg_attr" => nested_attributes(meta).iter().all(supported_attribute),
        "test" | "ignore" | "should_panic" | "cfg" | "allow" | "warn" | "deny" | "forbid"
        | "expect" | "doc" | "inline" | "cold" | "must_use" | "track_caller" => true,
        _ => false,
    }
}

fn attribute_diagnostics(attrs: &[Attribute]) -> Vec<Diagnostic> {
    attrs.iter().filter(|attr| !supported_attribute(&attr.meta)).map(|attr|
        Diagnostic::new(attr.pound_token.span,
            "unsupported function attribute; generated/parameterized tests need explicit checker support".into())
    ).collect()
}

#[derive(Default)]
struct BodyAudit {
    assertions: usize,
    diagnostics: Vec<Diagnostic>,
}

impl BodyAudit {
    fn extend(&mut self, other: Self) {
        self.assertions += other.assertions;
        self.diagnostics.extend(other.diagnostics);
    }
}

impl<'ast> Visit<'ast> for BodyAudit {
    // A nested function is checked separately, never counted as its parent's assertion.
    fn visit_item_fn(&mut self, _: &'ast ItemFn) {}
    fn visit_impl_item_fn(&mut self, _: &'ast syn::ImplItemFn) {}
    fn visit_trait_item_fn(&mut self, _: &'ast syn::TraitItemFn) {}
    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(segment) = node.path.segments.last() {
            self.extend(inspect_macro(
                &segment.ident.to_string(),
                segment.ident.span(),
                node.tokens.clone(),
            ));
        }
    }
}

fn inspect_body(body: &Block) -> BodyAudit {
    let mut audit = BodyAudit::default();
    audit.visit_block(body);
    audit
}

fn assertion_macro(name: &str) -> bool {
    matches!(
        name,
        "assert"
            | "assert_eq"
            | "assert_ne"
            | "debug_assert"
            | "debug_assert_eq"
            | "debug_assert_ne"
            | "prop_assert"
            | "prop_assert_eq"
            | "prop_assert_ne"
    )
}

fn transparent_macro(name: &str) -> bool {
    matches!(
        name,
        "vec"
            | "format"
            | "format_args"
            | "write"
            | "writeln"
            | "print"
            | "println"
            | "eprint"
            | "eprintln"
            | "matches"
            | "concat"
            | "env"
            | "option_env"
            | "include_str"
            | "include_bytes"
            | "stringify"
            | "file"
            | "line"
            | "column"
            | "module_path"
            | "cfg"
            | "panic"
            | "unreachable"
            | "todo"
            | "compile_error"
            | "prop_oneof"
    )
}

fn inspect_macro(name: &str, span: Span, tokens: TokenStream) -> BodyAudit {
    let mut audit = if matches!(name, "stringify" | "include_str" | "include_bytes") {
        BodyAudit::default()
    } else {
        inspect_tokens(tokens)
    };
    if assertion_macro(name) {
        audit.assertions += 1;
    } else if !transparent_macro(name) {
        audit.diagnostics.push(Diagnostic::new(
            span,
            format!("unsupported macro {name}!; assertions must not be hidden by custom macros"),
        ));
    }
    audit
}

// Token trees preserve groups and discard comments; literals are never parsed as source.
// Walk nested macro arguments so vec![{ assert!(...); ... }] cannot hide a second check.
fn inspect_tokens(tokens: TokenStream) -> BodyAudit {
    let tokens: Vec<_> = tokens.into_iter().collect();
    let mut audit = BodyAudit::default();
    let mut index = 0;
    while index < tokens.len() {
        if let (
            Some(TokenTree::Ident(name)),
            Some(TokenTree::Punct(bang)),
            Some(TokenTree::Group(args)),
        ) = (
            tokens.get(index),
            tokens.get(index + 1),
            tokens.get(index + 2),
        ) {
            if bang.as_char() == '!' {
                audit.extend(inspect_macro(&name.to_string(), name.span(), args.stream()));
                index += 3;
                continue;
            }
        }
        if let TokenTree::Group(group) = &tokens[index] {
            audit.extend(inspect_tokens(group.stream()));
        }
        index += 1;
    }
    audit
}
