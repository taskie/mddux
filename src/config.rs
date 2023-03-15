use std::collections::HashMap;

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
                special_comment_suffix: None,
            },
        );
        runners.insert(
            "bash".to_owned(),
            RunnerConfig {
                command: vec!["/bin/bash".to_owned()],
                name: Some("bash".to_owned()),
                special_comment_prefix: Some("#".to_owned()),
                special_comment_suffix: None,
            },
        );
        Config {
            caption: Some(true),
            runners,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub command: Vec<String>,
    pub name: Option<String>,
    pub special_comment_prefix: Option<String>,
    pub special_comment_suffix: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionConfig {
    // nop
}
