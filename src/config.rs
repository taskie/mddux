use std::{collections::HashMap, env::current_exe};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub caption: Option<bool>,
    pub runners: HashMap<String, RunnerConfig>,
}

impl Config {
    pub(crate) fn system_default() -> Self {
        let mut runners = HashMap::<String, RunnerConfig>::new();
        runners.insert(
            "sh".to_owned(),
            RunnerConfig {
                command: vec!["/bin/sh".to_owned()],
                name: Some("sh".to_owned()),
                special_comment_prefix: Some("#".to_owned()),
                ..Default::default()
            },
        );
        runners.insert(
            "bash".to_owned(),
            RunnerConfig {
                command: vec!["/bin/bash".to_owned()],
                name: Some("bash".to_owned()),
                special_comment_prefix: Some("#".to_owned()),
                ..Default::default()
            },
        );
        let mddux = current_exe().unwrap().to_str().unwrap().to_owned();
        runners.insert(
            "console".to_owned(),
            RunnerConfig {
                command: vec![mddux, "run-console".to_owned()],
                name: Some("console".to_owned()),
                special_comment_prefix: Some("$ #".to_owned()),
                stdout_info: Some("console".to_owned()),
                ..Default::default()
            },
        );
        Config {
            caption: Some(true),
            runners,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub command: Vec<String>,
    pub name: Option<String>,
    pub special_comment_prefix: Option<String>,
    pub special_comment_suffix: Option<String>,
    pub stdout_info: Option<String>,
    pub stderr_info: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionConfig {
    // nop
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormatConfig {
    pub stdout_info: Option<String>,
    pub stderr_info: Option<String>,
}

impl FormatConfig {
    pub(crate) fn apply_runner_config(&mut self, runner_config: &RunnerConfig) {
        if let Some(v) = runner_config.stdout_info.as_ref() {
            self.stdout_info = Some(v.clone())
        }
        if let Some(v) = runner_config.stderr_info.as_ref() {
            self.stderr_info = Some(v.clone())
        }
    }
}
