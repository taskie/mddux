use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{config::RunnerConfig, util::Content};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub runners: HashMap<String, RunnerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Execution {
    pub execution_count: i32,
    #[serde(rename = "type")]
    pub type_: String,
    pub input: ExecutionInput,
    pub output: Option<ExecutionOutput>,
}

impl Execution {
    pub(crate) fn execute(&mut self) -> Result<()> {
        self.output = Some(self.input.execute()?);
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionInput {
    pub command: ExecutionCommand,
    pub stdin_hash: i32,
    pub stdin: Content,
}

impl ExecutionInput {
    fn execute(&self) -> Result<ExecutionOutput> {
        let mut child = Command::new(&self.command.program)
            .args(&self.command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let child_stdin = child.stdin.as_mut().unwrap();
            child_stdin.write_all(self.stdin.as_ref()).unwrap();
        };
        let output = child.wait_with_output().unwrap();
        let result = ExecutionOutput {
            status_code: output.status.code(),
            stdout: Content::Binary(output.stdout.clone()),
            stderr: Content::Binary(output.stderr),
        };
        Ok(result)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionOutput {
    pub status_code: Option<i32>,
    pub stdout: Content,
    pub stderr: Content,
}
