use std::collections::HashMap;

use bstr::BString;
use serde::Serialize;

use crate::config::{ExecutionConfig, RunnerConfig};

#[derive(Debug, Serialize)]
pub struct Execution {
    pub execution_count: i32,
    pub type_: String,
    pub content: String,
    pub settings: ExecutionConfig,
    pub command: ExecutionCommand,
    pub result: ExecutionResult,
}

#[derive(Debug, Serialize)]
pub struct ExecutionCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub status_code: Option<i32>,
    pub stdout: BString,
    pub stderr: BString,
}

#[derive(Debug)]
pub struct ExecutionState {
    pub runners: HashMap<String, RunnerConfig>,
    pub executions: Vec<Execution>,
    pub execution_count: i32,
}
