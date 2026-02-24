use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::{ArgAction, Parser, Subcommand};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Parser)]
#[command(
    name = "switch",
    version,
    about = "Local-first context broker for coding agents"
)]
struct Cli {
    /// Override sqlite db path (default: ~/.switch/switch.db)
    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Ensure db exists and migrations are applied
    Init,
    /// Save a checkpoint for current repo + branch
    Save {
        #[arg(long)]
        done: Option<String>,
        #[arg(long)]
        next: Option<String>,
        #[arg(long)]
        blockers: Option<String>,
        #[arg(long)]
        tests: Option<String>,
        #[arg(long, action = ArgAction::Append)]
        files: Vec<String>,
        #[arg(long)]
        session: Option<String>,
    },
    /// Show latest checkpoint for current repo + branch
    Resume {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Show recent checkpoints for current repo + branch
    Log {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Run MCP stdio server for editor/agent integration
    McpServer,
}

#[derive(Debug, Serialize)]
struct Checkpoint {
    id: i64,
    repo_path: String,
    branch: String,
    commit_sha: String,
    session_id: Option<String>,
    done_text: Option<String>,
    next_text: Option<String>,
    blockers_text: Option<String>,
    tests_text: Option<String>,
    files: Vec<String>,
    created_at_ms: i64,
}

#[derive(Debug, Clone)]
struct ContextScope {
    repo_path: String,
    branch: String,
    commit_sha: String,
    used_repo_fallback: bool,
    used_branch_fallback: bool,
    used_commit_fallback: bool,
}

const JSON_RPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let conn = open_db(&db_path)?;

    match cli.command {
        Commands::Init => {
            println!("Initialized database at {}", db_path.display());
        }
        Commands::Save {
            done,
            next,
            blockers,
            tests,
            files,
            session,
        } => {
            let scope = detect_scope()?;
            warn_scope_fallback(&scope);
            save_checkpoint(&conn, &scope, done, next, blockers, tests, files, session)?;
            println!("Checkpoint saved");
        }
        Commands::Resume { json } => {
            let scope = detect_scope()?;
            warn_scope_fallback(&scope);
            if let Some(checkpoint) =
                latest_checkpoint_for_scope(&conn, &scope.repo_path, &scope.branch)?
            {
                if json {
                    println!("{}", serde_json::to_string_pretty(&checkpoint)?);
                } else {
                    print_checkpoint(&checkpoint);
                }
            } else {
                println!("No context found for this repo/branch.");
            }
        }
        Commands::Log { limit } => {
            let scope = detect_scope()?;
            warn_scope_fallback(&scope);
            let rows = list_checkpoints_for_scope(&conn, &scope.repo_path, &scope.branch, limit)?;
            if rows.is_empty() {
                println!("No checkpoints found.");
            } else {
                for row in rows {
                    print_checkpoint_compact(&row);
                }
            }
        }
        Commands::McpServer => {
            run_mcp_server(&conn)?;
        }
    }

    Ok(())
}

fn default_db_path() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home.join(".switch").join("switch.db"),
        None => PathBuf::from(".switch/switch.db"),
    }
}

fn open_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create db dir {}", parent.display()))?;
    }

    let conn = Connection::open(path)
        .with_context(|| format!("failed to open sqlite db at {}", path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(include_str!("../migrations/0001_init.sql"))?;
    Ok(conn)
}

fn save_checkpoint(
    conn: &Connection,
    scope: &ContextScope,
    done: Option<String>,
    next: Option<String>,
    blockers: Option<String>,
    tests: Option<String>,
    files: Vec<String>,
    session_id: Option<String>,
) -> Result<i64> {
    if done.is_none() && next.is_none() && blockers.is_none() && tests.is_none() && files.is_empty()
    {
        return Err(anyhow!(
            "at least one of --done, --next, --blockers, --tests, or --files is required"
        ));
    }

    let created_at_ms = current_time_ms()?;
    let files_json = serde_json::to_string(&files)?;

    conn.execute(
        "INSERT INTO checkpoints (
            repo_path, branch, commit_sha, session_id,
            done_text, next_text, blockers_text, tests_text, files_json, created_at_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            &scope.repo_path,
            &scope.branch,
            &scope.commit_sha,
            session_id,
            done,
            next,
            blockers,
            tests,
            files_json,
            created_at_ms
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn latest_checkpoint_for_scope(
    conn: &Connection,
    repo_path: &str,
    branch: &str,
) -> Result<Option<Checkpoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, branch, commit_sha, session_id, done_text, next_text, blockers_text, tests_text, files_json, created_at_ms
         FROM checkpoints
         WHERE repo_path = ?1 AND branch = ?2
         ORDER BY created_at_ms DESC, id DESC
         LIMIT 1",
    )?;

    let mut rows = stmt.query(params![repo_path, branch])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_checkpoint(row)?))
    } else {
        Ok(None)
    }
}

fn list_checkpoints_for_scope(
    conn: &Connection,
    repo_path: &str,
    branch: &str,
    limit: u32,
) -> Result<Vec<Checkpoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_path, branch, commit_sha, session_id, done_text, next_text, blockers_text, tests_text, files_json, created_at_ms
         FROM checkpoints
         WHERE repo_path = ?1 AND branch = ?2
         ORDER BY created_at_ms DESC, id DESC
         LIMIT ?3",
    )?;

    let mut rows = stmt.query(params![repo_path, branch, limit])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(row_to_checkpoint(row)?);
    }
    Ok(out)
}

fn row_to_checkpoint(row: &rusqlite::Row<'_>) -> Result<Checkpoint> {
    let id: i64 = row.get(0)?;
    let files_json: String = row.get(9)?;
    let files: Vec<String> = serde_json::from_str(&files_json)
        .with_context(|| format!("invalid files_json for checkpoint id {}", id))?;

    Ok(Checkpoint {
        id,
        repo_path: row.get(1)?,
        branch: row.get(2)?,
        commit_sha: row.get(3)?,
        session_id: row.get(4)?,
        done_text: row.get(5)?,
        next_text: row.get(6)?,
        blockers_text: row.get(7)?,
        tests_text: row.get(8)?,
        files,
        created_at_ms: row.get(10)?,
    })
}

fn print_checkpoint(c: &Checkpoint) {
    println!("repo: {}", c.repo_path);
    println!("branch: {}", c.branch);
    println!("commit: {}", c.commit_sha);
    println!("at: {}", format_ts(c.created_at_ms));
    if let Some(session_id) = &c.session_id {
        println!("session: {}", session_id);
    }
    if let Some(done) = &c.done_text {
        println!("done: {}", done);
    }
    if let Some(next) = &c.next_text {
        println!("next: {}", next);
    }
    if let Some(blockers) = &c.blockers_text {
        println!("blockers: {}", blockers);
    }
    if let Some(tests) = &c.tests_text {
        println!("tests: {}", tests);
    }
    if !c.files.is_empty() {
        println!("files: {}", c.files.join(", "));
    }
}

fn print_checkpoint_compact(c: &Checkpoint) {
    let done = truncate_for_log(c.done_text.as_deref().unwrap_or("-"), 96);
    let next = truncate_for_log(c.next_text.as_deref().unwrap_or("-"), 96);
    println!(
        "#{} [{}] done={} | next={}",
        c.id,
        format_ts(c.created_at_ms),
        done,
        next
    );
}

fn format_ts(ms: i64) -> String {
    match DateTime::<Utc>::from_timestamp_millis(ms) {
        Some(ts) => ts.to_rfc3339(),
        None => ms.to_string(),
    }
}

fn current_time_ms() -> Result<i64> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before unix epoch")?;
    Ok(i64::try_from(dur.as_millis()).context("timestamp overflow")?)
}

fn current_dir_fallback() -> Result<String> {
    let cwd = std::env::current_dir().context("failed to read current dir")?;
    Ok(cwd.to_string_lossy().to_string())
}

fn detect_scope() -> Result<ContextScope> {
    let repo_from_git = git_repo_root();
    let branch_from_git = git_value(["rev-parse", "--abbrev-ref", "HEAD"]);
    let commit_from_git = git_value(["rev-parse", "HEAD"]);

    let used_repo_fallback = repo_from_git.is_none();
    let used_branch_fallback = branch_from_git.is_none();
    let used_commit_fallback = commit_from_git.is_none();

    let repo_path = repo_from_git.unwrap_or(current_dir_fallback()?);
    let branch = branch_from_git.unwrap_or_else(|| "unknown".to_string());
    let commit_sha = commit_from_git.unwrap_or_else(|| "unknown".to_string());

    Ok(ContextScope {
        repo_path,
        branch,
        commit_sha,
        used_repo_fallback,
        used_branch_fallback,
        used_commit_fallback,
    })
}

fn warn_scope_fallback(scope: &ContextScope) {
    let mut reasons = Vec::new();
    if scope.used_repo_fallback {
        reasons.push("repo_path from current directory");
    }
    if scope.used_branch_fallback {
        reasons.push("branch set to 'unknown'");
    }
    if scope.used_commit_fallback {
        reasons.push("commit set to 'unknown'");
    }
    if reasons.is_empty() {
        return;
    }

    eprintln!(
        "warning: using fallback git scope ({}) for repo='{}', branch='{}'",
        reasons.join(", "),
        scope.repo_path,
        scope.branch
    );
}

fn truncate_for_log(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }

    let prefix: String = input.chars().take(max_chars - 3).collect();
    format!("{prefix}...")
}

fn git_repo_root() -> Option<String> {
    git_value(["rev-parse", "--show-toplevel"])
}

fn git_value<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let trimmed = s.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[derive(Debug)]
struct McpJsonRpcError {
    code: i64,
    message: String,
}

impl McpJsonRpcError {
    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {method}"),
        }
    }

    fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
        }
    }

    fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            code: -32000,
            message: message.into(),
        }
    }
}

fn run_mcp_server(conn: &Connection) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    while let Some(message) = read_mcp_message(&mut reader)? {
        match handle_mcp_message(conn, message) {
            Ok(Some(response)) => write_mcp_message(&mut writer, &response)?,
            Ok(None) => {}
            Err(err) => eprintln!("warning: failed to handle MCP message: {err:#}"),
        }
    }

    Ok(())
}

fn read_mcp_message<R: BufRead>(reader: &mut R) -> Result<Option<Value>> {
    let mut content_length: Option<usize> = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            if saw_header {
                return Err(anyhow!("unexpected EOF while reading MCP headers"));
            }
            return Ok(None);
        }
        saw_header = true;

        if line == "\n" || line == "\r\n" {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if let Some((name, value)) = trimmed.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length =
                    Some(value.trim().parse::<usize>().with_context(|| {
                        format!("invalid Content-Length header: {}", value.trim())
                    })?);
            }
        }
    }

    let length = content_length.ok_or_else(|| anyhow!("missing Content-Length header"))?;
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;

    let message: Value = serde_json::from_slice(&payload).context("invalid MCP JSON payload")?;
    Ok(Some(message))
}

fn write_mcp_message<W: Write>(writer: &mut W, payload: &Value) -> Result<()> {
    let body = serde_json::to_vec(payload)?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()?;
    Ok(())
}

fn handle_mcp_message(conn: &Connection, message: Value) -> Result<Option<Value>> {
    let message_obj = message
        .as_object()
        .ok_or_else(|| anyhow!("MCP message must be a JSON object"))?;

    let id = message_obj.get("id").cloned();
    let method = message_obj.get("method").and_then(Value::as_str);
    let params = message_obj
        .get("params")
        .cloned()
        .unwrap_or_else(|| json!({}));

    let method = match method {
        Some(method) => method,
        None => {
            if let Some(id) = id {
                let err = McpJsonRpcError::invalid_request("request is missing a string method");
                return Ok(Some(json!({
                    "jsonrpc": JSON_RPC_VERSION,
                    "id": id,
                    "error": {
                        "code": err.code,
                        "message": err.message,
                    }
                })));
            }
            return Ok(None);
        }
    };

    if let Some(id) = id {
        let response = match handle_mcp_request(conn, method, params) {
            Ok(result) => json!({
                "jsonrpc": JSON_RPC_VERSION,
                "id": id,
                "result": result
            }),
            Err(err) => json!({
                "jsonrpc": JSON_RPC_VERSION,
                "id": id,
                "error": {
                    "code": err.code,
                    "message": err.message,
                }
            }),
        };
        return Ok(Some(response));
    }

    handle_mcp_notification(method);
    Ok(None)
}

fn handle_mcp_notification(method: &str) {
    let _ = method;
}

fn handle_mcp_request(
    conn: &Connection,
    method: &str,
    params: Value,
) -> std::result::Result<Value, McpJsonRpcError> {
    match method {
        "initialize" => {
            let protocol_version = params
                .as_object()
                .and_then(|v| v.get("protocolVersion"))
                .and_then(Value::as_str)
                .unwrap_or(MCP_PROTOCOL_VERSION);

            Ok(json!({
                "protocolVersion": protocol_version,
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "switch-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }))
        }
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": mcp_tools(),
        })),
        "tools/call" => handle_mcp_tools_call(conn, params),
        _ => Err(McpJsonRpcError::method_not_found(method)),
    }
}

fn handle_mcp_tools_call(
    conn: &Connection,
    params: Value,
) -> std::result::Result<Value, McpJsonRpcError> {
    let params_obj = params
        .as_object()
        .ok_or_else(|| McpJsonRpcError::invalid_params("tools/call params must be an object"))?;
    let tool_name = required_string_arg(params_obj, "name")
        .map_err(|err| McpJsonRpcError::invalid_params(err.to_string()))?;
    let arguments = params_obj
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    if !arguments.is_object() {
        return Err(McpJsonRpcError::invalid_params(
            "tools/call arguments must be an object",
        ));
    }

    let tool_result = match tool_name.as_str() {
        "get_context" => mcp_get_context(conn, &arguments),
        "save_context" => mcp_save_context(conn, &arguments),
        "list_context" => mcp_list_context(conn, &arguments),
        _ => Err(anyhow!("unknown tool: {}", tool_name)),
    };

    match tool_result {
        Ok(structured_content) => mcp_tool_success(structured_content)
            .map_err(|err| McpJsonRpcError::internal(err.to_string())),
        Err(err) => Ok(mcp_tool_error(err)),
    }
}

fn mcp_tool_success(structured_content: Value) -> Result<Value> {
    Ok(json!({
        "content": [
            {
                "type": "text",
                "text": serde_json::to_string_pretty(&structured_content)?,
            }
        ],
        "structuredContent": structured_content
    }))
}

fn mcp_tool_error(err: anyhow::Error) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": format!("{err:#}"),
            }
        ],
        "isError": true
    })
}

fn mcp_get_context(conn: &Connection, arguments: &Value) -> Result<Value> {
    let args = arguments
        .as_object()
        .ok_or_else(|| anyhow!("get_context arguments must be an object"))?;
    let repo_path = required_string_arg(args, "repo_path")?;
    let branch = required_string_arg(args, "branch")?;

    let checkpoint = latest_checkpoint_for_scope(conn, &repo_path, &branch)?;
    if let Some(checkpoint) = checkpoint {
        Ok(json!({
            "found": true,
            "checkpoint": checkpoint_contract_object(&checkpoint),
        }))
    } else {
        Ok(json!({
            "found": false
        }))
    }
}

fn mcp_save_context(conn: &Connection, arguments: &Value) -> Result<Value> {
    let args = arguments
        .as_object()
        .ok_or_else(|| anyhow!("save_context arguments must be an object"))?;
    let repo_path = required_string_arg(args, "repo_path")?;
    let branch = required_string_arg(args, "branch")?;
    let commit_sha = required_string_arg(args, "commit_sha")?;
    let session_id = optional_string_arg(args, "session_id")?;
    let done_text = optional_string_arg(args, "done_text")?;
    let next_text = optional_string_arg(args, "next_text")?;
    let blockers_text = optional_string_arg(args, "blockers_text")?;
    let tests_text = optional_string_arg(args, "tests_text")?;
    let files = optional_string_list_arg(args, "files")?;

    let scope = ContextScope {
        repo_path,
        branch,
        commit_sha,
        used_repo_fallback: false,
        used_branch_fallback: false,
        used_commit_fallback: false,
    };
    let id = save_checkpoint(
        conn,
        &scope,
        done_text,
        next_text,
        blockers_text,
        tests_text,
        files,
        session_id,
    )?;

    Ok(json!({
        "ok": true,
        "id": id
    }))
}

fn mcp_list_context(conn: &Connection, arguments: &Value) -> Result<Value> {
    let args = arguments
        .as_object()
        .ok_or_else(|| anyhow!("list_context arguments must be an object"))?;
    let repo_path = required_string_arg(args, "repo_path")?;
    let branch = required_string_arg(args, "branch")?;
    let limit = optional_u32_arg(args, "limit", 20)?;
    if limit == 0 {
        return Err(anyhow!("limit must be at least 1"));
    }

    let items = list_checkpoints_for_scope(conn, &repo_path, &branch, limit)?
        .iter()
        .map(checkpoint_list_item)
        .collect::<Vec<Value>>();

    Ok(json!({
        "items": items
    }))
}

fn checkpoint_contract_object(checkpoint: &Checkpoint) -> Value {
    json!({
        "done_text": checkpoint.done_text,
        "next_text": checkpoint.next_text,
        "blockers_text": checkpoint.blockers_text,
        "tests_text": checkpoint.tests_text,
        "files": checkpoint.files,
        "commit_sha": checkpoint.commit_sha,
        "created_at_ms": checkpoint.created_at_ms
    })
}

fn checkpoint_list_item(checkpoint: &Checkpoint) -> Value {
    json!({
        "id": checkpoint.id,
        "repo_path": checkpoint.repo_path,
        "branch": checkpoint.branch,
        "session_id": checkpoint.session_id,
        "done_text": checkpoint.done_text,
        "next_text": checkpoint.next_text,
        "blockers_text": checkpoint.blockers_text,
        "tests_text": checkpoint.tests_text,
        "files": checkpoint.files,
        "commit_sha": checkpoint.commit_sha,
        "created_at_ms": checkpoint.created_at_ms
    })
}

fn required_string_arg(args: &Map<String, Value>, key: &str) -> Result<String> {
    match args.get(key) {
        Some(Value::String(v)) => Ok(v.clone()),
        Some(_) => Err(anyhow!("{key} must be a string")),
        None => Err(anyhow!("{key} is required")),
    }
}

fn optional_string_arg(args: &Map<String, Value>, key: &str) -> Result<Option<String>> {
    match args.get(key) {
        Some(Value::String(v)) => Ok(Some(v.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(anyhow!("{key} must be a string")),
    }
}

fn optional_string_list_arg(args: &Map<String, Value>, key: &str) -> Result<Vec<String>> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(Value::Array(values)) => values
            .iter()
            .enumerate()
            .map(|(i, value)| match value {
                Value::String(s) => Ok(s.clone()),
                _ => Err(anyhow!("{key}[{i}] must be a string")),
            })
            .collect(),
        Some(_) => Err(anyhow!("{key} must be an array of strings")),
    }
}

fn optional_u32_arg(args: &Map<String, Value>, key: &str, default: u32) -> Result<u32> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Number(n)) => {
            let as_u64 = n
                .as_u64()
                .ok_or_else(|| anyhow!("{key} must be a non-negative integer"))?;
            let as_u32 = u32::try_from(as_u64).context("value exceeds u32 range")?;
            Ok(as_u32)
        }
        Some(_) => Err(anyhow!("{key} must be a number")),
    }
}

fn mcp_tools() -> Value {
    json!([
        {
            "name": "get_context",
            "description": "Return the latest checkpoint for repo_path + branch.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "repo_path": { "type": "string" },
                    "branch": { "type": "string" }
                },
                "required": ["repo_path", "branch"]
            }
        },
        {
            "name": "save_context",
            "description": "Save a checkpoint for repo_path + branch.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "repo_path": { "type": "string" },
                    "branch": { "type": "string" },
                    "session_id": { "type": "string" },
                    "done_text": { "type": "string" },
                    "next_text": { "type": "string" },
                    "blockers_text": { "type": "string" },
                    "tests_text": { "type": "string" },
                    "files": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "commit_sha": { "type": "string" }
                },
                "required": ["repo_path", "branch", "commit_sha"]
            }
        },
        {
            "name": "list_context",
            "description": "List recent checkpoints for repo_path + branch.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "repo_path": { "type": "string" },
                    "branch": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1 }
                },
                "required": ["repo_path", "branch"]
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_db_path() -> PathBuf {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "switch-test-{}-{}-{}",
            std::process::id(),
            current_time_ms().expect("time"),
            test_id
        ));
        fs::create_dir_all(&base).expect("create temp dir");
        base.join("switch.db")
    }

    fn fixed_scope() -> ContextScope {
        ContextScope {
            repo_path: "/tmp/switch-test-repo".to_string(),
            branch: "feature/scope-tests".to_string(),
            commit_sha: "abc123".to_string(),
            used_repo_fallback: false,
            used_branch_fallback: false,
            used_commit_fallback: false,
        }
    }

    fn insert_checkpoint_raw(
        conn: &Connection,
        scope: &ContextScope,
        done: &str,
        created_at_ms: i64,
    ) -> i64 {
        conn.execute(
            "INSERT INTO checkpoints (
                repo_path, branch, commit_sha, done_text, files_json, created_at_ms
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &scope.repo_path,
                &scope.branch,
                &scope.commit_sha,
                done,
                "[]",
                created_at_ms
            ],
        )
        .expect("insert raw checkpoint");
        conn.last_insert_rowid()
    }

    #[test]
    fn init_creates_schema_and_is_idempotent() {
        let db_path = temp_db_path();
        let conn1 = open_db(&db_path).expect("open db first time");
        let order_index_exists: i64 = conn1
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_checkpoints_repo_branch_time_id'",
                [],
                |row| row.get(0),
            )
            .expect("query tie-break index");
        assert_eq!(order_index_exists, 1);
        let old_index_exists: i64 = conn1
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_checkpoints_repo_branch_time'",
                [],
                |row| row.get(0),
            )
            .expect("query old index");
        assert_eq!(old_index_exists, 0);

        conn1
            .execute(
                "INSERT INTO checkpoints (repo_path, branch, commit_sha, files_json, created_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["/tmp/a", "main", "abc", "[]", 1_i64],
            )
            .expect("insert after first init");
        conn1
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_checkpoints_repo_branch_time
                 ON checkpoints (repo_path, branch, created_at_ms DESC)",
                [],
            )
            .expect("seed old index");
        drop(conn1);

        let conn2 = open_db(&db_path).expect("open db second time");
        let count: i64 = conn2
            .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))
            .expect("count rows");
        assert_eq!(count, 1);
        let old_index_exists_after_reopen: i64 = conn2
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_checkpoints_repo_branch_time'",
                [],
                |row| row.get(0),
            )
            .expect("query old index after reopen");
        assert_eq!(old_index_exists_after_reopen, 0);
    }

    #[test]
    fn save_and_resume_round_trip() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");
        let scope = fixed_scope();

        save_checkpoint(
            &conn,
            &scope,
            Some("implemented parser".to_string()),
            Some("add tests".to_string()),
            Some("none".to_string()),
            Some("not run".to_string()),
            vec!["src/main.rs".to_string()],
            Some("claude-session".to_string()),
        )
        .expect("save checkpoint");

        let latest = latest_checkpoint_for_scope(&conn, &scope.repo_path, &scope.branch)
            .expect("query latest")
            .expect("checkpoint exists");

        assert_eq!(latest.repo_path.as_str(), scope.repo_path.as_str());
        assert_eq!(latest.branch.as_str(), scope.branch.as_str());
        assert_eq!(latest.commit_sha.as_str(), scope.commit_sha.as_str());
        assert_eq!(latest.done_text.as_deref(), Some("implemented parser"));
        assert_eq!(latest.next_text.as_deref(), Some("add tests"));
        assert_eq!(latest.files, vec!["src/main.rs"]);
        assert_eq!(latest.session_id.as_deref(), Some("claude-session"));
    }

    #[test]
    fn log_returns_desc_order_and_limit() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");
        let mut scope = fixed_scope();

        scope.commit_sha = "first".to_string();
        save_checkpoint(
            &conn,
            &scope,
            Some("first".to_string()),
            Some("first-next".to_string()),
            None,
            None,
            vec![],
            None,
        )
        .expect("first save");

        scope.commit_sha = "second".to_string();
        save_checkpoint(
            &conn,
            &scope,
            Some("second".to_string()),
            Some("second-next".to_string()),
            None,
            None,
            vec![],
            None,
        )
        .expect("second save");

        let logs = list_checkpoints_for_scope(&conn, &scope.repo_path, &scope.branch, 1)
            .expect("list logs");
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].done_text.as_deref(), Some("second"));
    }

    #[test]
    fn same_timestamp_uses_id_tiebreak_for_latest_and_log() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");
        let scope = fixed_scope();
        let ts = 123_456_789_i64;

        let first_id = insert_checkpoint_raw(&conn, &scope, "first", ts);
        let second_id = insert_checkpoint_raw(&conn, &scope, "second", ts);
        assert!(second_id > first_id);

        let latest = latest_checkpoint_for_scope(&conn, &scope.repo_path, &scope.branch)
            .expect("query latest")
            .expect("checkpoint exists");
        assert_eq!(latest.id, second_id);
        assert_eq!(latest.done_text.as_deref(), Some("second"));

        let logs = list_checkpoints_for_scope(&conn, &scope.repo_path, &scope.branch, 2)
            .expect("list logs");
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].id, second_id);
        assert_eq!(logs[0].done_text.as_deref(), Some("second"));
        assert_eq!(logs[1].id, first_id);
        assert_eq!(logs[1].done_text.as_deref(), Some("first"));
    }

    #[test]
    fn save_requires_at_least_one_payload_field() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");
        let scope = fixed_scope();

        let err = save_checkpoint(&conn, &scope, None, None, None, None, vec![], None)
            .expect_err("save should fail");
        let msg = format!("{err:#}");
        assert!(msg.contains(
            "at least one of --done, --next, --blockers, --tests, or --files is required"
        ));
    }

    #[test]
    fn invalid_files_json_returns_error() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");
        let scope = fixed_scope();

        conn.execute(
            "INSERT INTO checkpoints (
                repo_path, branch, commit_sha, done_text, files_json, created_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &scope.repo_path,
                &scope.branch,
                &scope.commit_sha,
                "bad row",
                "{not-valid-json",
                current_time_ms().expect("time")
            ],
        )
        .expect("insert malformed row");

        let err = latest_checkpoint_for_scope(&conn, &scope.repo_path, &scope.branch)
            .expect_err("expected parse error");
        let msg = format!("{err:#}");
        assert!(msg.contains("invalid files_json for checkpoint id"));
    }

    #[test]
    fn mcp_tools_list_includes_context_tools() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");

        let response =
            handle_mcp_request(&conn, "tools/list", json!({})).expect("tools/list should succeed");
        let tools = response["tools"]
            .as_array()
            .expect("tools should be an array");
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .collect();

        assert!(names.contains(&"get_context"));
        assert!(names.contains(&"save_context"));
        assert!(names.contains(&"list_context"));
    }

    #[test]
    fn mcp_save_then_get_and_list_round_trip() {
        let db_path = temp_db_path();
        let conn = open_db(&db_path).expect("open db");

        let save = handle_mcp_request(
            &conn,
            "tools/call",
            json!({
                "name": "save_context",
                "arguments": {
                    "repo_path": "/tmp/mcp-repo",
                    "branch": "feature/mcp",
                    "session_id": "claude-1",
                    "done_text": "wired MCP",
                    "next_text": "test integrations",
                    "blockers_text": "none",
                    "tests_text": "cargo test",
                    "files": ["src/main.rs"],
                    "commit_sha": "abc123"
                }
            }),
        )
        .expect("save_context should succeed");

        assert_ne!(save.get("isError").and_then(Value::as_bool), Some(true));
        let save_payload = &save["structuredContent"];
        assert_eq!(save_payload["ok"], true);
        assert!(
            save_payload["id"].as_i64().expect("id should be i64") > 0,
            "id should be positive"
        );

        let get = handle_mcp_request(
            &conn,
            "tools/call",
            json!({
                "name": "get_context",
                "arguments": {
                    "repo_path": "/tmp/mcp-repo",
                    "branch": "feature/mcp"
                }
            }),
        )
        .expect("get_context should succeed");

        let get_payload = &get["structuredContent"];
        assert_eq!(get_payload["found"], true);
        assert_eq!(get_payload["checkpoint"]["done_text"], "wired MCP");
        assert_eq!(get_payload["checkpoint"]["next_text"], "test integrations");
        assert_eq!(get_payload["checkpoint"]["commit_sha"], "abc123");

        let list = handle_mcp_request(
            &conn,
            "tools/call",
            json!({
                "name": "list_context",
                "arguments": {
                    "repo_path": "/tmp/mcp-repo",
                    "branch": "feature/mcp",
                    "limit": 10
                }
            }),
        )
        .expect("list_context should succeed");

        let items = list["structuredContent"]["items"]
            .as_array()
            .expect("items should be an array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["done_text"], "wired MCP");
    }

    #[test]
    fn truncate_for_log_applies_ellipsis() {
        assert_eq!(truncate_for_log("short", 10), "short");
        assert_eq!(
            truncate_for_log("abcdefghijklmnopqrstuvwxyz", 8),
            "abcde..."
        );
        assert_eq!(truncate_for_log("abcdef", 3), "...");
    }
}
