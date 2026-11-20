//! Skill validation logic.

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::constants::{ALLOWED_FIELDS, CLAUDE_FIELDS};
use crate::parser::{find_skill_md, parse_frontmatter};
use crate::yaml::{FmValue, Frontmatter};

pub const MAX_SKILL_NAME_LENGTH: usize = 64;
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;
pub const MAX_COMPATIBILITY_LENGTH: usize = 500;

/// NFKC-normalize a string (matches Python's `unicodedata.normalize("NFKC", ...)`).
fn nfkc(s: &str) -> String {
    s.nfkc().collect()
}

/// Number of Unicode scalar values, matching Python's `len(str)`.
fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Validate skill name format and directory match.
///
/// Skill names support i18n characters (Unicode letters) plus hyphens. Names
/// must be lowercase and cannot start or end with a hyphen.
fn validate_name(name: &FmValue, skill_dir: Option<&Path>) -> Vec<String> {
    let Some(raw) = name.as_str().filter(|s| !s.trim().is_empty()) else {
        return vec!["Field 'name' must be a non-empty string".to_string()];
    };

    let mut errors = Vec::new();

    let name = nfkc(raw.trim());

    if char_len(&name) > MAX_SKILL_NAME_LENGTH {
        errors.push(format!(
            "Skill name '{name}' exceeds {MAX_SKILL_NAME_LENGTH} character limit ({} chars)",
            char_len(&name)
        ));
    }

    if name != name.to_lowercase() {
        errors.push(format!("Skill name '{name}' must be lowercase"));
    }

    if name.starts_with('-') || name.ends_with('-') {
        errors.push("Skill name cannot start or end with a hyphen".to_string());
    }

    if name.contains("--") {
        errors.push("Skill name cannot contain consecutive hyphens".to_string());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        errors.push(format!(
            "Skill name '{name}' contains invalid characters. \
             Only letters, digits, and hyphens are allowed."
        ));
    }

    if let Some(dir) = skill_dir {
        let dir_name_raw = dir
            .file_name()
            .map_or_default(|n| n.to_string_lossy().into_owned());
        let dir_name = nfkc(&dir_name_raw);
        if dir_name != name {
            errors.push(format!(
                "Directory name '{dir_name_raw}' must match skill name '{name}'"
            ));
        }
    }

    errors
}

/// Validate description format.
fn validate_description(description: &FmValue) -> Vec<String> {
    let Some(desc) = description.as_str().filter(|s| !s.trim().is_empty()) else {
        return vec!["Field 'description' must be a non-empty string".to_string()];
    };

    let mut errors = Vec::new();

    if char_len(desc) > MAX_DESCRIPTION_LENGTH {
        errors.push(format!(
            "Description exceeds {MAX_DESCRIPTION_LENGTH} character limit ({} chars)",
            char_len(desc)
        ));
    }

    errors
}

/// Validate compatibility format.
fn validate_compatibility(compatibility: &FmValue) -> Vec<String> {
    let Some(compat) = compatibility.as_str() else {
        return vec!["Field 'compatibility' must be a string".to_string()];
    };

    let mut errors = Vec::new();

    if char_len(compat) > MAX_COMPATIBILITY_LENGTH {
        errors.push(format!(
            "Compatibility exceeds {MAX_COMPATIBILITY_LENGTH} character limit ({} chars)",
            char_len(compat)
        ));
    }

    errors
}

/// Validate that only allowed fields are present.
///
/// When `allow_claude_fields` is set, the Claude Code [`CLAUDE_FIELDS`] join the
/// base [`ALLOWED_FIELDS`] for both the membership check and the error message.
fn validate_metadata_fields(metadata: &Frontmatter, allow_claude_fields: bool) -> Vec<String> {
    let mut allowed = BTreeSet::from(ALLOWED_FIELDS);
    if allow_claude_fields {
        allowed.extend(CLAUDE_FIELDS);
    }

    // Both sides iterate in sorted order (BTreeSet), matching Python's
    // `sorted(extra_fields)` / `sorted(ALLOWED_FIELDS)` in the message.
    let keys = metadata.keys();
    let extra: Vec<&str> = keys
        .iter()
        .map(String::as_str)
        .filter(|k| !allowed.contains(k))
        .collect();

    if extra.is_empty() {
        return Vec::new();
    }

    let allowed_repr = fmt::from_fn(|f| {
        for (i, field) in allowed.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "'{field}'")?;
        }
        Ok(())
    });
    vec![format!(
        "Unexpected fields in frontmatter: {}. Only [{allowed_repr}] are allowed.",
        extra.join(", "),
    )]
}

/// Validate parsed skill metadata.
///
/// This is the core validation routine that works on already-parsed
/// frontmatter, avoiding duplicate file I/O. Returns the list of validation
/// error messages; an empty list means valid.
///
/// When `allow_claude_fields` is set, Claude Code's [`CLAUDE_FIELDS`] are
/// permitted in addition to the base-spec [`ALLOWED_FIELDS`].
pub fn validate_metadata(
    metadata: &Frontmatter,
    skill_dir: Option<&Path>,
    allow_claude_fields: bool,
) -> Vec<String> {
    let mut errors = Vec::new();
    errors.extend(validate_metadata_fields(metadata, allow_claude_fields));

    match metadata.get("name") {
        None => errors.push("Missing required field in frontmatter: name".to_string()),
        Some(name) => errors.extend(validate_name(name, skill_dir)),
    }

    match metadata.get("description") {
        None => errors.push("Missing required field in frontmatter: description".to_string()),
        Some(description) => errors.extend(validate_description(description)),
    }

    if let Some(compatibility) = metadata.get("compatibility") {
        errors.extend(validate_compatibility(compatibility));
    }

    errors
}

/// Validate a skill directory.
///
/// Returns the list of validation error messages; an empty list means the
/// skill is valid. When `allow_claude_fields` is set, Claude Code's
/// [`CLAUDE_FIELDS`] are permitted in addition to the base-spec
/// [`ALLOWED_FIELDS`].
pub fn validate(skill_dir: &Path, allow_claude_fields: bool) -> Vec<String> {
    if !skill_dir.exists() {
        return vec![format!("Path does not exist: {}", skill_dir.display())];
    }

    if !skill_dir.is_dir() {
        return vec![format!("Not a directory: {}", skill_dir.display())];
    }

    let Some(skill_md) = find_skill_md(skill_dir) else {
        return vec!["Missing required file: SKILL.md".to_string()];
    };

    let content = match std::fs::read_to_string(&skill_md) {
        Ok(c) => c,
        Err(e) => return vec![format!("Could not read {}: {e}", skill_md.display())],
    };

    match parse_frontmatter(&content) {
        Ok((metadata, _body)) => validate_metadata(&metadata, Some(skill_dir), allow_claude_fields),
        Err(e) => vec![e.to_string()],
    }
}
