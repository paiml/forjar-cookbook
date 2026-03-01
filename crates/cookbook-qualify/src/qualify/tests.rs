//! Tests for recipe qualification types.

use super::*;

#[test]
fn status_from_csv_qualified() {
    assert_eq!(
        RecipeStatus::from_csv("qualified"),
        Ok(RecipeStatus::Qualified)
    );
}

#[test]
fn status_from_csv_blocked() {
    assert_eq!(RecipeStatus::from_csv("blocked"), Ok(RecipeStatus::Blocked));
}

#[test]
fn status_from_csv_pending() {
    assert_eq!(RecipeStatus::from_csv("pending"), Ok(RecipeStatus::Pending));
}

#[test]
fn status_from_csv_case_insensitive() {
    assert_eq!(
        RecipeStatus::from_csv("QUALIFIED"),
        Ok(RecipeStatus::Qualified)
    );
    assert_eq!(RecipeStatus::from_csv("Blocked"), Ok(RecipeStatus::Blocked));
}

#[test]
fn status_from_csv_with_whitespace() {
    assert_eq!(
        RecipeStatus::from_csv("  pending  "),
        Ok(RecipeStatus::Pending)
    );
}

#[test]
fn status_from_csv_unknown() {
    assert!(RecipeStatus::from_csv("invalid").is_err());
}

#[test]
fn status_badge_qualified() {
    assert!(RecipeStatus::Qualified.badge().contains("QUALIFIED"));
    assert!(RecipeStatus::Qualified.badge().contains("brightgreen"));
}

#[test]
fn status_badge_blocked() {
    assert!(RecipeStatus::Blocked.badge().contains("BLOCKED"));
    assert!(RecipeStatus::Blocked.badge().contains("red"));
}

#[test]
fn status_badge_pending() {
    assert!(RecipeStatus::Pending.badge().contains("PENDING"));
    assert!(RecipeStatus::Pending.badge().contains("lightgray"));
}

#[test]
fn status_as_str_roundtrip() {
    for status in &[
        RecipeStatus::Qualified,
        RecipeStatus::Blocked,
        RecipeStatus::Pending,
    ] {
        let s = status.as_str();
        let parsed = RecipeStatus::from_csv(s);
        assert_eq!(parsed, Ok(status.clone()));
    }
}

#[test]
fn idempotency_from_csv_strong() {
    assert_eq!(
        IdempotencyClass::from_csv("strong"),
        Ok(IdempotencyClass::Strong)
    );
}

#[test]
fn idempotency_from_csv_weak() {
    assert_eq!(
        IdempotencyClass::from_csv("weak"),
        Ok(IdempotencyClass::Weak)
    );
}

#[test]
fn idempotency_from_csv_eventual() {
    assert_eq!(
        IdempotencyClass::from_csv("eventual"),
        Ok(IdempotencyClass::Eventual)
    );
}

#[test]
fn idempotency_from_csv_unknown() {
    assert!(IdempotencyClass::from_csv("invalid").is_err());
}

#[test]
fn idempotency_as_str_roundtrip() {
    for class in &[
        IdempotencyClass::Strong,
        IdempotencyClass::Weak,
        IdempotencyClass::Eventual,
    ] {
        let s = class.as_str();
        let parsed = IdempotencyClass::from_csv(s);
        assert_eq!(parsed, Ok(class.clone()));
    }
}

#[test]
fn recipe_qualification_serde_roundtrip() {
    let qual = RecipeQualification {
        recipe_num: 1,
        name: "developer-workstation".to_string(),
        category: "infra".to_string(),
        status: RecipeStatus::Qualified,
        tier: "2+3".to_string(),
        idempotency_class: IdempotencyClass::Strong,
        first_apply_ms: 45000,
        idempotent_apply_ms: 1200,
        blocker_ticket: String::new(),
        blocker_description: String::new(),
        last_qualified: "2026-03-01".to_string(),
        qualified_by: "cookbook-runner".to_string(),
    };
    let json = serde_json::to_string(&qual).unwrap_or_default();
    assert!(json.contains("developer-workstation"));
    assert!(json.contains("qualified"));
}
