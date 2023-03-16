use std::collections::HashMap;

use bstr::BString;
use serde::Serialize;

use crate::config::{ExecutionConfig, RunnerConfig};

#[derive(Clone, Debug)]
pub struct ExecutionState {
    pub environment: ExecutionEnvironment,
    pub executions: Vec<Execution>,
    pub execution_count: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionEnvironment {
    pub runners: HashMap<String, RunnerConfig>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Execution {
    pub execution_count: i32,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
    pub config: ExecutionConfig,
    pub command: ExecutionCommand,
    pub result: Option<ExecutionResult>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionResult {
    pub status_code: Option<i32>,
    pub stdout: BString,
    pub stderr: BString,
}
