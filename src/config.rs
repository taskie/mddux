use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runners: Option<HashMap<String, RunnerConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<bool>,
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
        // If the full path of the running binary is embedded in the state file,
        // it might cause compatibility issues across different environments.
        // Therefore, we're using the string "$MDDUX" here.
        let mddux = "$MDDUX".to_owned();
        runners.insert(
            "console".to_owned(),
            RunnerConfig {
                command: vec![mddux, "run-console".to_owned()],
                name: Some("console".to_owned()),
                special_comment_prefix: Some("$ #".to_owned()),
                format: Some(FormatConfig {
                    stdout_info: Some("console".to_owned()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        Config {
            caption: Some(true),
            runners: Some(runners),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub command: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_comment_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_comment_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exection: Option<ExecutionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatConfig>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<bool>,
}

macro_rules! apply_config {
    ($self: expr, $target: expr, $field: ident) => {
        if let Some(v) = $target.$field.as_ref() {
            $self.$field = Some(v.clone())
        }
    };
}

impl ExecutionConfig {
    pub(crate) fn apply(&mut self, other: &ExecutionConfig) {
        apply_config!(self, other, skipped);
    }

    pub(crate) fn insert_with_str(&mut self, key: &str, value: &str) -> bool {
        match key {
            "skipped" => {
                self.skipped = Some(value.eq_ignore_ascii_case("true"));
            }
            _ => return false,
        }
        true
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FormatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_caption_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdin_caption_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_caption_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_caption_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout_info: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_caption_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_caption_hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_info: Option<String>,
}

impl FormatConfig {
    pub(crate) fn apply(&mut self, other: &FormatConfig) {
        apply_config!(self, other, stdin_hidden);
        apply_config!(self, other, stdin_caption_format);
        apply_config!(self, other, stdin_caption_hidden);
        apply_config!(self, other, stdout_hidden);
        apply_config!(self, other, stdout_caption_format);
        apply_config!(self, other, stdout_caption_hidden);
        apply_config!(self, other, stdout_info);
        apply_config!(self, other, stderr_hidden);
        apply_config!(self, other, stderr_caption_format);
        apply_config!(self, other, stderr_caption_hidden);
        apply_config!(self, other, stderr_info);
    }

    pub(crate) fn insert_with_str(&mut self, key: &str, value: &str) -> bool {
        match key {
            "stdin-hidden" => {
                self.stdin_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stdin-caption-format" => {
                self.stdin_caption_format = Some(value.to_owned());
            }
            "stdin-caption-hidden" => {
                self.stdin_caption_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stdout-hidden" => {
                self.stdout_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stdout-caption-format" => {
                self.stdout_caption_format = Some(value.to_owned());
            }
            "stdout-caption-hidden" => {
                self.stdout_caption_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stdout-info" => {
                self.stdout_info = Some(value.to_owned());
            }
            "stderr-hidden" => {
                self.stderr_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stderr-caption-format" => {
                self.stderr_caption_format = Some(value.to_owned());
            }
            "stderr-caption-hidden" => {
                self.stderr_caption_hidden = Some(value.eq_ignore_ascii_case("true"));
            }
            "stderr-info" => {
                self.stderr_info = Some(value.to_owned());
            }
            _ => return false,
        }
        true
    }
}
