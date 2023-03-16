use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
};

use anyhow::Result;
use bstr::BString;
use serde::{Deserialize, Serialize};

use crate::config::{Config, FormatConfig, RunnerConfig};

#[derive(Clone, Debug)]
pub struct Executor {
    pub state: ExecutionState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionState {
    pub environment: ExecutionEnvironment,
    pub executions: Vec<Execution>,
    pub code_block_to_execution: HashMap<usize, usize>,
    pub front_matter: Option<Vec<u8>>,
    pub format_configs: Vec<FormatConfig>,
}

impl ExecutionState {
    pub(crate) fn from_config(config: &Config) -> ExecutionState {
        ExecutionState {
            environment: ExecutionEnvironment {
                runners: config.runners.clone(),
            },
            executions: vec![],
            code_block_to_execution: HashMap::new(),
            front_matter: None,
            format_configs: vec![],
        }
    }

    pub(crate) fn execute_all(&mut self) -> Result<()> {
        for exe in self.executions.iter_mut() {
            exe.execute()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub runners: HashMap<String, RunnerConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Execution {
    pub execution_count: i32,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: String,
    pub input: ExecutionInput,
    pub output: Option<ExecutionOutput>,
}

impl Execution {
    pub(crate) fn execute(&mut self) -> Result<()> {
        self.output = Some(self.input.execute()?);
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionInput {
    pub command: ExecutionCommand,
    pub stdin: BString,
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
            child_stdin.write_all(&self.stdin).unwrap();
        };
        let output = child.wait_with_output().unwrap();
        let result = ExecutionOutput {
            status_code: output.status.code(),
            stdout: BString::new(output.stdout.clone()),
            stderr: BString::new(output.stderr),
        };
        Ok(result)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionOutput {
    pub status_code: Option<i32>,
    pub stdout: BString,
    pub stderr: BString,
}
