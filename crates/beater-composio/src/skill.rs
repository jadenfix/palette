//! Prompting scaffold ("skill cards") around Composio tools.
//!
//! A bare tool slug is useless to an agent — it can't know what the tool does,
//! when to reach for it, or how to shape the arguments. This module turns a
//! [`ConnectorTool`]'s metadata (description, tags, input JSON Schema) into:
//!
//! * [`skill_card`] — a human/agent-readable markdown block for one tool, with
//!   a *when to use* hint, the argument contract, and the exact Beater MCP call
//!   (`invokeConnectorTool`) to run it.
//! * [`skills_doc`] — many cards grouped by toolkit, the large "skills.md"
//!   surface that grows as more Composio tools are adopted, ready to splice into
//!   an agent's system prompt.
//! * [`tool_definition_json`] — the `tools.json` *entry* the RSI loop's
//!   `apply_change`/`ToolAdd` writes into an agent repo, so a tool addition
//!   lands schema-and-hint-complete rather than as a naked slug.
//!
//! Everything here is derived from Composio's own metadata (no invented facts),
//! so it stays correct as Composio updates the catalog.

use serde_json::{Value, json};

use crate::ConnectorTool;

/// The MCP/`/v1` operation an agent calls to run a connector tool. Kept as a
/// constant so the scaffold and the API contract can't drift in wording.
pub const INVOKE_OPERATION: &str = "invokeConnectorTool";

/// Render a single tool as a markdown skill card.
pub fn skill_card(tool: &ConnectorTool) -> String {
    let mut out = String::new();
    out.push_str(&format!("### {} (`{}`)\n", tool.name, tool.slug));
    if let Some(desc) = tool.description.as_deref().filter(|d| !d.is_empty()) {
        out.push_str(desc.trim());
        out.push_str("\n\n");
    }
    let toolkit = tool.toolkit.as_deref().unwrap_or("composio");
    let auth = if tool.no_auth {
        "no connection required"
    } else {
        "requires a connected account (run the connect flow once)"
    };
    out.push_str(&format!("- **Toolkit:** `{toolkit}` · **Auth:** {auth}\n"));
    if !tool.tags.is_empty() {
        out.push_str(&format!("- **Tags:** {}\n", tool.tags.join(", ")));
    }
    out.push_str(&format!("- **When to use:** {}\n", when_to_use(tool)));

    let args = render_arguments(tool.input_schema.as_ref());
    if args.is_empty() {
        out.push_str("- **Arguments:** none\n");
    } else {
        out.push_str("- **Arguments:**\n");
        for line in &args {
            out.push_str(&format!("  - {line}\n"));
        }
    }
    out.push_str(&format!(
        "- **Invoke:** `{INVOKE_OPERATION}` with `{{ \"tool\": \"{}\", \"arguments\": {{ … }} }}`\n",
        tool.slug
    ));
    out
}

/// Assemble many skill cards into one document, grouped by toolkit, with a
/// header explaining the contract to the agent. This is the "skills.md" surface.
pub fn skills_doc(tools: &[ConnectorTool]) -> String {
    let mut out = String::from(
        "# Connector tools (Composio)\n\nYou can call any tool below through the \
         Beater MCP tool `invokeConnectorTool` (or `POST /v1/connectors/{tenant}/{project}/invoke`). \
         Pass the tool's `slug` and an `arguments` object matching its schema. If a tool \
         requires a connected account and none exists, first request the one-time login link \
         via `connectConnector`.\n\n",
    );
    // Stable, grouped ordering keeps the generated doc deterministic (important
    // for snapshot/drift checks and reproducible prompts).
    let mut by_toolkit: std::collections::BTreeMap<&str, Vec<&ConnectorTool>> =
        std::collections::BTreeMap::new();
    for tool in tools {
        let key = tool.toolkit.as_deref().unwrap_or("composio");
        by_toolkit.entry(key).or_default().push(tool);
    }
    for (toolkit, mut group) in by_toolkit {
        group.sort_by(|a, b| a.slug.cmp(&b.slug));
        out.push_str(&format!("## {toolkit}\n\n"));
        for tool in group {
            out.push_str(&skill_card(tool));
            out.push('\n');
        }
    }
    out
}

/// Build the `tools.json` entry the RSI loop writes when adding a tool to an
/// agent's `tool_set` (the §6.1 lever). Superset of the harness's
/// `{name, description, symbol}` shape, plus the data needed to actually call
/// and document the tool. `symbol` encodes the invocation so an agent reading
/// `tools.json` knows the entry point.
pub fn tool_definition_json(tool: &ConnectorTool) -> Value {
    json!({
        "name": tool.slug,
        "description": tool.description.clone().unwrap_or_else(|| tool.name.clone()),
        "symbol": format!("{INVOKE_OPERATION}({})", tool.slug),
        "source": "composio",
        "toolkit": tool.toolkit.clone().unwrap_or_else(|| "composio".to_string()),
        "no_auth": tool.no_auth,
        "input_schema": tool.input_schema.clone().unwrap_or_else(|| json!({})),
        "skill_card": skill_card(tool),
    })
}

/// Derive a short "when to use" hint from the tool's metadata. Honest: it only
/// reshapes the description/tags Composio provides.
fn when_to_use(tool: &ConnectorTool) -> String {
    if let Some(desc) = tool.description.as_deref().filter(|d| !d.is_empty()) {
        // First sentence of the description is the most actionable hint.
        let first = desc.split(['.', '\n']).next().unwrap_or(desc).trim();
        if !first.is_empty() {
            return format!("{first}.");
        }
    }
    match tool.toolkit.as_deref() {
        Some(tk) => format!("When the task needs `{tk}`."),
        None => "When the task needs this capability.".to_string(),
    }
}

/// Flatten a JSON Schema `properties`/`required` object into per-argument
/// bullet lines: `` `name` (type, required): description ``.
fn render_arguments(schema: Option<&Value>) -> Vec<String> {
    let Some(schema) = schema else {
        return Vec::new();
    };
    let required: std::collections::BTreeSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    // BTreeMap iteration over the serde_json Map is insertion-ordered; sort for
    // determinism.
    let mut names: Vec<&String> = props.keys().collect();
    names.sort();
    for name in names {
        let spec = &props[name];
        let ty = spec.get("type").and_then(Value::as_str).unwrap_or("any");
        let req = if required.contains(name.as_str()) {
            "required"
        } else {
            "optional"
        };
        let desc = spec
            .get("description")
            .and_then(Value::as_str)
            .map(|d| format!(" — {d}"))
            .unwrap_or_default();
        lines.push(format!("`{name}` ({ty}, {req}){desc}"));
    }
    lines
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn github_issue_tool() -> ConnectorTool {
        ConnectorTool {
            slug: "GITHUB_CREATE_AN_ISSUE".to_string(),
            name: "Create an issue".to_string(),
            description: Some(
                "Create a new issue in a GitHub repository. Use for filing bugs.".to_string(),
            ),
            no_auth: false,
            toolkit: Some("github".to_string()),
            tags: vec!["important".to_string()],
            input_schema: Some(json!({
                "type": "object",
                "required": ["owner", "repo", "title"],
                "properties": {
                    "owner": {"type": "string", "description": "Repo owner"},
                    "repo": {"type": "string", "description": "Repo name"},
                    "title": {"type": "string", "description": "Issue title"},
                    "body": {"type": "string", "description": "Issue body"}
                }
            })),
        }
    }

    #[test]
    fn skill_card_has_hint_args_and_invocation() {
        let card = skill_card(&github_issue_tool());
        assert!(card.contains("GITHUB_CREATE_AN_ISSUE"));
        assert!(card.contains("When to use:"));
        // arguments are listed with required/optional + description
        assert!(card.contains("`title` (string, required) — Issue title"));
        assert!(card.contains("`body` (string, optional) — Issue body"));
        // the exact invocation contract is surfaced
        assert!(card.contains("invokeConnectorTool"));
        assert!(card.contains("requires a connected account"));
    }

    #[test]
    fn no_arg_tool_says_none() {
        let mut t = github_issue_tool();
        t.input_schema = None;
        assert!(skill_card(&t).contains("**Arguments:** none"));
    }

    #[test]
    fn skills_doc_groups_by_toolkit_deterministically() {
        let a = ConnectorTool {
            slug: "SLACK_SEND".to_string(),
            name: "Send".to_string(),
            description: None,
            no_auth: false,
            toolkit: Some("slack".to_string()),
            tags: vec![],
            input_schema: None,
        };
        let doc1 = skills_doc(&[github_issue_tool(), a.clone()]);
        let doc2 = skills_doc(&[a, github_issue_tool()]);
        // Grouped + sorted → input order doesn't change output (snapshot-safe).
        assert_eq!(doc1, doc2);
        assert!(doc1.contains("## github"));
        assert!(doc1.contains("## slack"));
        assert!(doc1.find("## github").unwrap() < doc1.find("## slack").unwrap());
    }

    #[test]
    fn tool_definition_is_rsi_tools_json_shape() {
        let def = tool_definition_json(&github_issue_tool());
        // Harness-compatible core fields.
        assert_eq!(def["name"], "GITHUB_CREATE_AN_ISSUE");
        assert!(def["description"].as_str().unwrap().contains("issue"));
        assert_eq!(def["symbol"], "invokeConnectorTool(GITHUB_CREATE_AN_ISSUE)");
        // Enrichment for a complete ToolAdd.
        assert_eq!(def["source"], "composio");
        assert_eq!(def["toolkit"], "github");
        assert_eq!(def["input_schema"]["properties"]["title"]["type"], "string");
        assert!(def["skill_card"].as_str().unwrap().contains("When to use:"));
    }

    #[test]
    fn when_to_use_falls_back_to_toolkit() {
        let mut t = github_issue_tool();
        t.description = None;
        assert!(when_to_use(&t).contains("github"));
    }
}
