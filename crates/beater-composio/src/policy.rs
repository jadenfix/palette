use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use utoipa::ToSchema;

use crate::ConnectorTool;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorToolRiskClass {
    ReadOnly,
    ExternalWrite,
    Destructive,
    Messaging,
    Payment,
    SecretAccess,
    Unknown,
}

impl ConnectorToolRiskClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ExternalWrite => "external_write",
            Self::Destructive => "destructive",
            Self::Messaging => "messaging",
            Self::Payment => "payment",
            Self::SecretAccess => "secret_access",
            Self::Unknown => "unknown",
        }
    }

    pub fn requires_explicit_allow(self) -> bool {
        !matches!(self, Self::ReadOnly)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorToolPolicyDecision {
    pub allowed: bool,
    pub risk_class: ConnectorToolRiskClass,
    pub reason: String,
    pub approval_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectorToolPolicy {
    allowed_tools: BTreeSet<String>,
    denied_tools: BTreeSet<String>,
    allow_read_only_by_default: bool,
}

impl Default for ConnectorToolPolicy {
    fn default() -> Self {
        Self {
            allowed_tools: BTreeSet::new(),
            denied_tools: BTreeSet::new(),
            allow_read_only_by_default: true,
        }
    }
}

impl ConnectorToolPolicy {
    pub fn with_allowed_tools<I, S>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_tools.extend(normalize_tool_slugs(tools));
        self
    }

    pub fn with_denied_tools<I, S>(mut self, tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.denied_tools.extend(normalize_tool_slugs(tools));
        self
    }

    pub fn allowed_tool_count(&self) -> usize {
        self.allowed_tools.len()
    }

    pub fn denied_tool_count(&self) -> usize {
        self.denied_tools.len()
    }

    pub fn evaluate(&self, tool: &ConnectorTool) -> ConnectorToolPolicyDecision {
        let slug = normalize_tool_slug(&tool.slug);
        let risk_class = classify_connector_tool(tool);

        if self.denied_tools.contains(&slug) {
            return ConnectorToolPolicyDecision {
                allowed: false,
                risk_class,
                reason: "tool is explicitly denied by connector policy".to_string(),
                approval_id: None,
            };
        }

        if self.allowed_tools.contains(&slug) {
            return ConnectorToolPolicyDecision {
                allowed: true,
                risk_class,
                reason: "tool is explicitly allowed by connector policy".to_string(),
                approval_id: Some(format!("configured-allowlist:{slug}")),
            };
        }

        if risk_class == ConnectorToolRiskClass::ReadOnly && self.allow_read_only_by_default {
            return ConnectorToolPolicyDecision {
                allowed: true,
                risk_class,
                reason: "read-only connector tools are allowed by default".to_string(),
                approval_id: None,
            };
        }

        ConnectorToolPolicyDecision {
            allowed: false,
            risk_class,
            reason: format!(
                "risk class {} requires an explicit connector allowlist entry",
                risk_class.as_str()
            ),
            approval_id: None,
        }
    }
}

pub fn classify_connector_tool(tool: &ConnectorTool) -> ConnectorToolRiskClass {
    let slug = tool.slug.to_ascii_lowercase();
    let words = slug_words(&slug);
    let haystack = searchable_text(tool);

    if contains_any(&haystack, &PAYMENT_KEYWORDS) || contains_word(&words, &PAYMENT_WORDS) {
        return ConnectorToolRiskClass::Payment;
    }
    if contains_any(&haystack, &SECRET_KEYWORDS) || contains_word(&words, &SECRET_WORDS) {
        return ConnectorToolRiskClass::SecretAccess;
    }
    if contains_any(&haystack, &DESTRUCTIVE_KEYWORDS) || contains_word(&words, &DESTRUCTIVE_WORDS) {
        return ConnectorToolRiskClass::Destructive;
    }
    if contains_any(&haystack, &MESSAGING_KEYWORDS) || contains_word(&words, &MESSAGING_WORDS) {
        return ConnectorToolRiskClass::Messaging;
    }
    if contains_any(&haystack, &WRITE_KEYWORDS) || contains_word(&words, &WRITE_WORDS) {
        return ConnectorToolRiskClass::ExternalWrite;
    }
    if contains_any(&haystack, &READ_KEYWORDS) || contains_word(&words, &READ_WORDS) {
        return ConnectorToolRiskClass::ReadOnly;
    }

    ConnectorToolRiskClass::Unknown
}

fn normalize_tool_slugs<I, S>(tools: I) -> impl Iterator<Item = String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    tools
        .into_iter()
        .map(normalize_tool_slug)
        .filter(|slug| !slug.is_empty())
}

fn normalize_tool_slug(slug: impl Into<String>) -> String {
    slug.into().trim().to_ascii_uppercase()
}

fn searchable_text(tool: &ConnectorTool) -> String {
    let mut text = String::new();
    text.push_str(&tool.slug);
    text.push(' ');
    text.push_str(&tool.name);
    text.push(' ');
    if let Some(description) = &tool.description {
        text.push_str(description);
        text.push(' ');
    }
    for tag in &tool.tags {
        text.push_str(tag);
        text.push(' ');
    }
    text.to_ascii_lowercase()
}

fn slug_words(slug: &str) -> BTreeSet<&str> {
    slug.split(['_', '-', ' ', ':', '.'])
        .filter(|word| !word.is_empty())
        .collect()
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn contains_word(words: &BTreeSet<&str>, needles: &[&str]) -> bool {
    needles.iter().any(|needle| words.contains(needle))
}

const PAYMENT_KEYWORDS: [&str; 10] = [
    "payment",
    "checkout",
    "charge",
    "refund",
    "invoice",
    "subscription",
    "billing",
    "payout",
    "stripe",
    "bank account",
];
const PAYMENT_WORDS: [&str; 9] = [
    "payment",
    "pay",
    "checkout",
    "charge",
    "refund",
    "invoice",
    "subscription",
    "billing",
    "payout",
];

const SECRET_KEYWORDS: [&str; 9] = [
    "secret",
    "token",
    "password",
    "credential",
    "api key",
    "apikey",
    "private key",
    "oauth",
    "access key",
];
const SECRET_WORDS: [&str; 8] = [
    "secret",
    "token",
    "password",
    "credential",
    "credentials",
    "apikey",
    "oauth",
    "key",
];

const DESTRUCTIVE_KEYWORDS: [&str; 9] = [
    "delete",
    "remove",
    "revoke",
    "disable",
    "destroy",
    "drop",
    "terminate",
    "cancel",
    "permanently",
];
const DESTRUCTIVE_WORDS: [&str; 8] = [
    "delete",
    "remove",
    "revoke",
    "disable",
    "destroy",
    "drop",
    "terminate",
    "cancel",
];

const MESSAGING_KEYWORDS: [&str; 10] = [
    "send message",
    "send email",
    "send sms",
    "post message",
    "reply",
    "comment",
    "tweet",
    "slack",
    "discord",
    "telegram",
];
const MESSAGING_WORDS: [&str; 10] = [
    "send", "email", "message", "sms", "reply", "comment", "tweet", "slack", "discord", "telegram",
];

const WRITE_KEYWORDS: [&str; 14] = [
    "create",
    "update",
    "write",
    "edit",
    "patch",
    "publish",
    "upload",
    "merge",
    "commit",
    "push",
    "invite",
    "assign",
    "submit",
    "open a new",
];
const WRITE_WORDS: [&str; 14] = [
    "create", "update", "write", "edit", "patch", "publish", "upload", "merge", "commit", "push",
    "add", "invite", "assign", "submit",
];

const READ_KEYWORDS: [&str; 11] = [
    "get", "list", "read", "fetch", "search", "find", "retrieve", "query", "lookup", "inspect",
    "describe",
];
const READ_WORDS: [&str; 11] = [
    "get", "list", "read", "fetch", "search", "find", "retrieve", "query", "lookup", "inspect",
    "describe",
];

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tool(slug: &str, description: &str) -> ConnectorTool {
        ConnectorTool {
            slug: slug.to_string(),
            name: slug.replace('_', " "),
            description: Some(description.to_string()),
            no_auth: false,
            toolkit: Some("github".to_string()),
            tags: vec![],
            input_schema: Some(json!({"type": "object"})),
        }
    }

    #[test]
    fn classifies_common_connector_risks() {
        assert_eq!(
            classify_connector_tool(&tool("GITHUB_GET_REPOSITORY", "Get repository metadata")),
            ConnectorToolRiskClass::ReadOnly
        );
        assert_eq!(
            classify_connector_tool(&tool("GITHUB_CREATE_AN_ISSUE", "Open a new issue")),
            ConnectorToolRiskClass::ExternalWrite
        );
        assert_eq!(
            classify_connector_tool(&tool("SLACK_SEND_MESSAGE", "Send message to a channel")),
            ConnectorToolRiskClass::Messaging
        );
        assert_eq!(
            classify_connector_tool(&tool("GITHUB_DELETE_REPOSITORY", "Delete a repository")),
            ConnectorToolRiskClass::Destructive
        );
        assert_eq!(
            classify_connector_tool(&tool("STRIPE_CREATE_CHARGE", "Create a payment charge")),
            ConnectorToolRiskClass::Payment
        );
        assert_eq!(
            classify_connector_tool(&tool("VAULT_GET_SECRET", "Read a secret value")),
            ConnectorToolRiskClass::SecretAccess
        );
    }

    #[test]
    fn default_policy_allows_read_only_and_denies_write_tools() {
        let policy = ConnectorToolPolicy::default();
        assert!(
            policy
                .evaluate(&tool("GITHUB_GET_REPOSITORY", "Get repository metadata"))
                .allowed
        );
        let decision = policy.evaluate(&tool("GITHUB_CREATE_AN_ISSUE", "Open a new issue"));
        assert!(!decision.allowed);
        assert_eq!(decision.risk_class, ConnectorToolRiskClass::ExternalWrite);
    }

    #[test]
    fn explicit_allowlist_permits_high_risk_tools_and_denylist_wins() {
        let create_issue = tool("GITHUB_CREATE_AN_ISSUE", "Open a new issue");
        let policy = ConnectorToolPolicy::default()
            .with_allowed_tools(["github_create_an_issue"])
            .with_denied_tools(["GITHUB_CREATE_AN_ISSUE"]);
        let decision = policy.evaluate(&create_issue);
        assert!(!decision.allowed);
        assert!(decision.reason.contains("explicitly denied"));

        let policy = ConnectorToolPolicy::default().with_allowed_tools(["github_create_an_issue"]);
        let decision = policy.evaluate(&create_issue);
        assert!(decision.allowed);
        assert_eq!(
            decision.approval_id.as_deref(),
            Some("configured-allowlist:GITHUB_CREATE_AN_ISSUE")
        );
    }
}
