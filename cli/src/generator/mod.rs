//! Generator module for creating llms.txt and SKILL.md files.
//!
//! This module provides:
//! - `LlmsTxtGenerator` for generating llms.txt documentation
//! - `SkillGenerator` for generating SKILL.md files from llms.txt

mod llms;
mod skill;
mod templates;

pub use llms::{LlmsTxt, LlmsTxtGenerator};
pub use skill::SkillGenerator;
