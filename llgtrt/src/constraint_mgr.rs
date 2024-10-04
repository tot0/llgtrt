use crate::routes::api_ext::LlgLogLevel;
use anyhow::Result;
use llguidance_parser::{
    api::{ParserLimits, TopLevelGrammar},
    Constraint, Logger, TokenParser,
};
use serde::{Deserialize, Serialize};
use toktrie::{InferenceCapabilities, TokEnv};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LlgConfig {
    /// Override any of the parser limits.
    pub limits: ParserLimits,

    /// Log level which goes to stderr. In-memory logs per-sequence are managed by ConstraintInit.log_level.
    pub log_level: Option<u32>,
}

pub struct ConstraintInit {
    pub grammar: TopLevelGrammar,
    pub is_chat: bool,
    pub log_level: LlgLogLevel,
}

pub struct ConstraintMgr {
    tok_env: TokEnv,
    chat_tok_env: TokEnv,
    inference_caps: InferenceCapabilities,
    parser_limits: ParserLimits,
    log_stderr_level: u32,
}

impl ConstraintMgr {
    pub fn new(
        tok_env: TokEnv,
        chat_tok_env: TokEnv,
        mut config: serde_json::Value,
    ) -> Result<Self> {
        let defl_limits = serde_json::to_value(ParserLimits::default()).unwrap();
        if let Some(obj) = config["limits"].as_object_mut() {
            for (k, v) in defl_limits.as_object().unwrap() {
                if !obj.contains_key(k) {
                    obj.insert(k.clone(), v.clone());
                }
            }
        } else {
            config["limits"] = defl_limits;
        }
        let config: LlgConfig = serde_json::from_value(config)?;

        Ok(ConstraintMgr {
            tok_env,
            chat_tok_env,
            inference_caps: InferenceCapabilities {
                ff_tokens: false, // not supported yet
                backtrack: false, // unlikely
                ..Default::default()
            },
            parser_limits: config.limits,
            log_stderr_level: config.log_level.unwrap_or(1),
        })
    }

    pub fn new_constraint(&self, init: ConstraintInit) -> Result<Constraint> {
        let parser = TokenParser::from_llguidance_json(
            if init.is_chat {
                self.chat_tok_env.clone()
            } else {
                self.tok_env.clone()
            },
            init.grammar,
            Logger::new(init.log_level.to_log_level(), self.log_stderr_level),
            self.inference_caps.clone(),
            self.parser_limits.clone(),
            vec![],
        )?;
        let mut constraint = Constraint::new(parser);
        if init.log_level.has_json() {
            constraint.log_json_progress = true;
        }
        Ok(constraint)
    }

    #[allow(dead_code)]
    pub fn tok_trie(&self) -> &toktrie::TokTrie {
        self.tok_env.tok_trie()
    }
}