use fence::policy::model::Policy;
use fence::policy::parser::parse;

#[test]
fn parses_empty_policy() {
    let policy = parse("").unwrap();
    assert_eq!(policy, Policy::default());
}

#[test]
fn parses_sections() {
    let input = r#"
        [filesystem]
        
        [process]

        [network]
    "#;

    let policy = parse(input).unwrap();

    assert_eq!(policy, Policy::default());
}

#[test]
fn ignores_blank_lines_and_comments() {
    let input = r#"
        # Fence policy

        [filesystem]

        # Process rules will come later

        [process]

        [network]
    "#;

    assert_eq!(parse(input).unwrap(), Policy::default());
}

#[test]
fn rejects_unknown_section() {
    let error = parse("[unknown]").unwrap_err();

    assert_eq!(error.line, 1);
}

#[test]
fn rejects_rule_before_section() {
    let error = parse("allow read /tmp/**").unwrap_err();

    assert_eq!(error.line, 1);
}

#[test]
fn rejects_rules_for_now() {
    let error = parse(
        r#"
        [filesystem]
        allow read /tmp/**
        "#,
    )
    .unwrap_err();

    assert_eq!(error.line, 3);
}
