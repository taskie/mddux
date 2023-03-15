use std::{collections::HashMap, env::current_exe};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub caption: Option<bool>,
    pub runners: HashMap<String, RunnerConfig>,
}

impl Default for Config {
    fn default() -> Self {
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

#[derive(Debug, Serialize)]
pub struct ExecutionConfig {
    // nop
}
