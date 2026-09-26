//! Policy contract checks; each test owns one assertion and helpers return setup/results.

use restqs_test_policy::analyze;

/// Analyze a source fixture and return diagnostic messages for one focused assertion.
fn messages(source: &str) -> Result<Vec<String>, syn::Error> {
    Ok(analyze(source)?
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect())
}

#[test]
fn accepts_one_assertion() -> Result<(), syn::Error> {
    assert!(messages("#[test] fn works() { assert_eq!(1, 1); }")?.is_empty());
    Ok(())
}

#[test]
fn rejects_zero_assertions() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] fn empty() {}")?,
        ["test empty: expected exactly one assertion, found 0"]
    );
    Ok(())
}

#[test]
fn reports_test_location() -> Result<(), syn::Error> {
    let report = analyze("\n#[test]\nfn empty() {}")?;
    assert_eq!(
        (report.diagnostics[0].line, report.diagnostics[0].column),
        (3, 4)
    );
    Ok(())
}

#[test]
fn rejects_multiple_assertions_on_one_line() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] fn many() { assert!(true); assert_ne!(1, 2); }")?,
        ["test many: expected exactly one assertion, found 2"]
    );
    Ok(())
}

#[test]
fn ignores_comments_and_literals() -> Result<(), syn::Error> {
    let source = r##"#[test] fn text() { /* assert!(false); */ let s = r#"assert!(false)"#; assert_eq!(s, "assert!(false)"); }"##;
    assert!(messages(source)?.is_empty());
    Ok(())
}

#[test]
fn recognizes_namespaced_async_test() -> Result<(), syn::Error> {
    assert_eq!(
        analyze("#[tokio::test] async fn future() { std::assert!(true); }")?.tests,
        1
    );
    Ok(())
}

#[test]
fn recognizes_nested_conditional_test_attribute() -> Result<(), syn::Error> {
    assert_eq!(
        analyze(
            "#[cfg_attr(test, cfg_attr(feature = \"x\", test))] fn conditional() { assert!(true); }"
        )?
        .tests,
        1
    );
    Ok(())
}

#[test]
fn checks_disabled_tests() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[cfg(any())] #[test] fn disabled() {}")?,
        ["test disabled: expected exactly one assertion, found 0"]
    );
    Ok(())
}

#[test]
fn checks_ignored_tests() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] #[ignore] fn ignored() {}")?,
        ["test ignored: expected exactly one assertion, found 0"]
    );
    Ok(())
}

#[test]
fn rejects_helper_assertions() -> Result<(), syn::Error> {
    assert_eq!(
        messages("fn setup() { assert!(true); }")?,
        ["helper setup: assertions belong in test functions, found 1"]
    );
    Ok(())
}

#[test]
fn nested_helper_does_not_satisfy_outer_test() -> Result<(), syn::Error> {
    assert!(
        messages("#[test] fn outer() { fn helper() { assert!(true); } }")?
            .contains(&"test outer: expected exactly one assertion, found 0".to_owned())
    );
    Ok(())
}

#[test]
fn rejects_assertion_in_helper_method() -> Result<(), syn::Error> {
    assert_eq!(
        messages("impl Setup { fn run(&self) { assert!(true); } }")?,
        ["helper run: assertions belong in test functions, found 1"]
    );
    Ok(())
}

#[test]
fn sees_assertions_in_macro_arguments() -> Result<(), syn::Error> {
    assert_eq!(
        messages(
            "#[test] fn nested() { let v = vec![{ assert!(true); 1 }]; assert_eq!(v, vec![1]); }"
        )?,
        ["test nested: expected exactly one assertion, found 2"]
    );
    Ok(())
}

#[test]
fn sees_assertion_in_closure() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] fn closure() { let check = || assert!(true); check(); assert!(true); }")?,
        ["test closure: expected exactly one assertion, found 2"]
    );
    Ok(())
}

#[test]
fn stringified_code_does_not_count() -> Result<(), syn::Error> {
    assert!(
        messages(
            "#[test] fn text() { let s = stringify!(assert!(false)); assert!(!s.is_empty()); }"
        )?
        .is_empty()
    );
    Ok(())
}

#[test]
fn debug_assertions_count() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] fn debug() { debug_assert!(true); assert!(true); }")?,
        ["test debug: expected exactly one assertion, found 2"]
    );
    Ok(())
}

#[test]
fn property_assertions_count() -> Result<(), syn::Error> {
    assert_eq!(
        messages("#[test] fn property() { prop_assert_eq!(1, 1); prop_assert_ne!(1, 2); }")?,
        ["test property: expected exactly one assertion, found 2"]
    );
    Ok(())
}

#[test]
fn rejects_opaque_assertion_macro() -> Result<(), syn::Error> {
    assert!(
        messages("#[test] fn hidden() { check_it!(); assert!(true); }")?
            .iter()
            .any(|m| m.contains("unsupported macro check_it!"))
    );
    Ok(())
}

#[test]
fn rejects_generated_item_tests() -> Result<(), syn::Error> {
    assert!(
        messages("generate_tests!();")?
            .iter()
            .any(|m| m.contains("generated tests require explicit checker support"))
    );
    Ok(())
}

#[test]
fn rejects_macro_definitions_that_can_hide_tests() -> Result<(), syn::Error> {
    assert!(!messages("macro_rules! tests { () => { #[test] fn hidden() {} } }")?.is_empty());
    Ok(())
}

#[test]
fn rejects_parameterized_test_attributes() -> Result<(), syn::Error> {
    assert!(
        messages("#[test_case(1)] fn generated(value: i32) { assert!(value > 0); }")?
            .iter()
            .any(|m| m.contains("unsupported function attribute"))
    );
    Ok(())
}

#[test]
fn rejects_invalid_syntax() {
    assert!(analyze("#[test] fn invalid(").is_err());
}

#[test]
fn checks_fuzz_target_assertions() -> Result<(), syn::Error> {
    assert_eq!(
        messages("fuzz_target!(|data: &[u8]| { consume(data); });")?,
        ["test fuzz_target: expected exactly one assertion, found 0"]
    );
    Ok(())
}

#[test]
fn accepts_fuzz_target_with_one_assertion() -> Result<(), syn::Error> {
    assert!(messages("fuzz_target!(|data: &[u8]| { assert!(valid(data)); });")?.is_empty());
    Ok(())
}
