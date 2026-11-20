//! CLI for the `skref` library — a Rust port of `skills-ref`.

// clap renders `///` doc comments verbatim as `--help` text, so backticks
// would show up literally in the CLI output.
#![allow(clippy::doc_markdown)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use skref::{read_properties, to_prompt, validate};

/// Reference library for Agent Skills.
#[derive(Parser)]
#[command(name = "skref", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a skill directory.
    ///
    /// Checks that the skill has a valid SKILL.md with proper frontmatter,
    /// correct naming conventions, and required fields.
    Validate {
        /// Path to a skill directory (or directly to its SKILL.md).
        skill_path: PathBuf,

        /// Also accept Claude Code's extra frontmatter fields (model,
        /// when_to_use, argument-hint, hooks, …) instead of rejecting them.
        #[arg(long)]
        allow_claude_fields: bool,
    },

    /// Read and print skill properties as JSON.
    ReadProperties {
        /// Path to a skill directory (or directly to its SKILL.md).
        skill_path: PathBuf,

        /// Also include Claude Code's extra frontmatter fields (model,
        /// when_to_use, argument-hint, hooks, …) in the JSON output.
        #[arg(long)]
        allow_claude_fields: bool,
    },

    /// Generate `<available_skills>` XML for agent prompts.
    ToPrompt {
        /// One or more skill directories (or paths to SKILL.md files).
        #[arg(required = true)]
        skill_paths: Vec<PathBuf>,
    },
}

/// Whether `path` points directly to a `SKILL.md`/`skill.md` file.
fn is_skill_md_file(path: &Path) -> bool {
    path.is_file()
        && path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().to_lowercase() == "skill.md")
}

/// If `path` is a SKILL.md file, return its parent directory; otherwise return
/// `path` unchanged.
fn resolve_skill_path(path: &Path) -> PathBuf {
    if is_skill_md_file(path)
        && let Some(parent) = path.parent()
    {
        parent.to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn cmd_validate(skill_path: &Path, allow_claude_fields: bool) -> ExitCode {
    let skill_path = resolve_skill_path(skill_path);
    let errors = validate(&skill_path, allow_claude_fields);

    if errors.is_empty() {
        println!("Valid skill: {}", skill_path.display());
        ExitCode::SUCCESS
    } else {
        eprintln!("Validation failed for {}:", skill_path.display());
        for error in &errors {
            eprintln!("  - {error}");
        }
        ExitCode::FAILURE
    }
}

fn cmd_read_properties(skill_path: &Path, allow_claude_fields: bool) -> ExitCode {
    let skill_path = resolve_skill_path(skill_path);
    match read_properties(&skill_path, allow_claude_fields) {
        Ok(props) => {
            println!("{}", props.to_json());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_to_prompt(skill_paths: &[PathBuf]) -> ExitCode {
    let resolved: Vec<PathBuf> = skill_paths.iter().map(|p| resolve_skill_path(p)).collect();
    match to_prompt(&resolved) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Validate {
            skill_path,
            allow_claude_fields,
        } => cmd_validate(&skill_path, allow_claude_fields),
        Command::ReadProperties {
            skill_path,
            allow_claude_fields,
        } => cmd_read_properties(&skill_path, allow_claude_fields),
        Command::ToPrompt { skill_paths } => cmd_to_prompt(&skill_paths),
    }
}
