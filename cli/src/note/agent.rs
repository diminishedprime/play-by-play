#![allow(missing_docs)]

use std::borrow::Cow;
use std::io::Write;

use serde::Deserialize;
use serde_json::Value;

use crate::error::AppResult;
use crate::log::{CLEAR, Color, RESET, Screen};
use crate::markdown::Markdown;
use crate::note::{Body, Header, Open};

#[derive(Deserialize)]
pub struct AgentInput<'a> {
    #[serde(borrow)]
    pub description: Cow<'a, str>,
    #[serde(borrow)]
    pub prompt: Cow<'a, str>,
    #[serde(borrow)]
    pub subagent_type: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub model: Option<Open<'a, Model>>,
    #[serde(borrow)]
    pub effort: Option<Open<'a, Effort>>,
    pub run_in_background: Option<bool>,
    #[serde(borrow)]
    pub name: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub team_name: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub mode: Option<Open<'a, PermissionMode>>,
    #[serde(borrow)]
    pub isolation: Option<Open<'a, Isolation>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Sonnet,
    Opus,
    Haiku,
    Fable,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    AcceptEdits,
    Auto,
    BypassPermissions,
    Default,
    DontAsk,
    Plan,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Isolation {
    Worktree,
    Remote,
}

#[expect(
    clippy::large_enum_variant,
    reason = "it lives on the stack and is used once, so its size isn't worth boxing"
)]
#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AgentResult<'a> {
    Completed(#[serde(borrow)] AgentAnswer<'a>),
    AsyncLaunched(#[serde(borrow)] AgentLaunch<'a>),
    RemoteLaunched(#[serde(borrow)] AgentRemote<'a>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAnswer<'a> {
    #[serde(borrow)]
    pub agent_id: Cow<'a, str>,
    pub harness_note_count: Option<u64>,
    pub harness_tail_count: Option<u64>,
    #[serde(borrow)]
    pub harness_section_hash: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub agent_type: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub handback: Option<Open<'a, Handback>>,
    #[serde(borrow)]
    pub handback_report: Option<HandbackReport<'a>>,
    #[serde(borrow)]
    pub content: Vec<AgentText<'a>>,
    #[serde(borrow)]
    pub resolved_model: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub models_used: Option<Vec<Cow<'a, str>>>,
    pub total_tool_use_count: u64,
    pub total_duration_ms: f64,
    pub total_tokens: u64,
    #[serde(borrow)]
    pub usage: Usage<'a>,
    #[serde(borrow)]
    pub tools_used: Option<Vec<Cow<'a, str>>>,
    pub tool_stats: Option<ToolStats>,
    #[serde(borrow)]
    pub prompt: Cow<'a, str>,
    #[serde(borrow)]
    pub worktree_path: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub worktree_branch: Option<Cow<'a, str>>,
    pub can_continue_agent: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Handback {
    Send,
    Flagged,
    Withheld,
}

#[derive(Deserialize)]
pub struct HandbackReport<'a> {
    #[serde(borrow)]
    pub text: Cow<'a, str>,
    #[serde(borrow)]
    pub warning: Option<Cow<'a, str>>,
}

#[derive(Deserialize)]
pub struct AgentText<'a> {
    #[serde(borrow)]
    pub text: Cow<'a, str>,
    pub citations: Option<Vec<Value>>,
}

#[derive(Deserialize)]
pub struct Usage<'a> {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub server_tool_use: Option<ServerToolUse>,
    #[serde(borrow)]
    pub service_tier: Option<Cow<'a, str>>,
    pub cache_creation: Option<CacheCreation>,
    #[serde(borrow)]
    pub inference_geo: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub speed: Option<Cow<'a, str>>,
    pub iterations: Option<Value>,
    pub output_tokens_details: Option<OutputTokensDetails>,
    pub fallback_credit: Option<Value>,
}

#[derive(Deserialize)]
pub struct ServerToolUse {
    pub web_search_requests: u64,
    pub web_fetch_requests: u64,
}

#[derive(Deserialize)]
pub struct CacheCreation {
    pub ephemeral_1h_input_tokens: u64,
    pub ephemeral_5m_input_tokens: u64,
}

#[derive(Deserialize)]
pub struct OutputTokensDetails {
    pub thinking_tokens: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStats {
    pub read_count: u64,
    pub search_count: u64,
    pub bash_count: u64,
    pub edit_file_count: u64,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub other_tool_count: u64,
    pub frame_count: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentLaunch<'a> {
    pub is_async: Option<bool>,
    #[serde(borrow)]
    pub agent_id: Cow<'a, str>,
    #[serde(borrow)]
    pub description: Cow<'a, str>,
    #[serde(borrow)]
    pub resolved_model: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub models_used: Option<Vec<Cow<'a, str>>>,
    #[serde(borrow)]
    pub prompt: Cow<'a, str>,
    #[serde(borrow)]
    pub output_file: Cow<'a, str>,
    pub can_read_output_file: Option<bool>,
    pub can_continue_agent: Option<bool>,
    pub shares_cwd: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRemote<'a> {
    #[serde(borrow)]
    pub task_id: Cow<'a, str>,
    #[serde(borrow)]
    pub session_url: Cow<'a, str>,
    #[serde(borrow)]
    pub description: Cow<'a, str>,
    #[serde(borrow)]
    pub prompt: Cow<'a, str>,
    #[serde(borrow)]
    pub output_file: Cow<'a, str>,
}

impl Header for AgentInput<'_> {
    fn header(&self, out: &mut impl Write, color: Color, screen: &Screen) -> AppResult {
        let name = self.subagent_type.as_deref().unwrap_or(&self.description);
        writeln!(out, "{CLEAR}{color}agent: {name}\nprompt:{RESET}")?;
        Markdown::write(&self.prompt, &mut *out, screen)
    }
}

impl Body for AgentResult<'_> {
    fn body(&self, out: &mut impl Write, screen: &Screen) -> AppResult {
        match self {
            AgentResult::Completed(answer) => {
                writeln!(out, "response:")?;
                for block in &answer.content {
                    Markdown::write(&block.text, &mut *out, screen)?;
                }
            }
            AgentResult::AsyncLaunched(launch) => {
                writeln!(out, "running in the background: {}", launch.output_file)?;
            }
            AgentResult::RemoteLaunched(remote) => {
                writeln!(out, "running in the cloud: {}", remote.session_url)?;
            }
        }
        Ok(())
    }
}
