# Development

`skref` is a Rust port of the Python [`skills-ref`](https://github.com/agentskills/agentskills/tree/main/skills-ref) reference library. Behavior should stay faithful to that library and to the [Agent Skills specification](https://github.com/agentskills/agentskills/blob/main/docs/specification.mdx).

## Layout

- `src/lib.rs` — public API re-exports.
- `src/main.rs` — the `skref` CLI (clap).
- `src/parser.rs` — `find_skill_md`, `parse_frontmatter`, `read_properties`.
- `src/validator.rs` — `validate` / `validate_metadata` and the field rules.
- `src/prompt.rs` — `to_prompt` (`<available_skills>` XML).
- `src/models.rs` — `SkillProperties` + JSON output.
- `src/yaml.rs` — frontmatter value model (parsed with `yaml_serde`, the maintained `serde_yaml` successor). Mirrors `strictyaml`: every scalar is coerced to a string.
- `src/errors.rs` — `SkillError` (Parse / Validation).
- `tests/` — `parser.rs`, `prompt.rs`, `validator.rs` are 1:1 ports of the Python tests; `sample_skills.rs` exercises the on-disk fixtures.
- `tests/fixtures/` — `sample-skills/` (base-spec valid), `invalid-skills/` (intentionally broken), `claude-skills/` (valid only with `--allow-claude-fields`; `claude-everything` uses every supported field).

## Quality

```bash
cargo fmt --all
CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets
CARGO_BUILD_WARNINGS=deny cargo test
```

- MSRV is the latest stable Rust (`rust-version` in `Cargo.toml`); feel free to use
  any stable language/std feature.
- Lint policy lives in `Cargo.toml`'s `[lints]` table (rustc, clippy pedantic, and
  Cargo's own lints such as `unused_dependencies`).
- `.cargo/config.toml` sets a 7-day dependency cooldown
  (`registry.global-min-publish-age`): `cargo update` ignores versions published in
  the last week unless they are already in `Cargo.lock`.

## Releases

A release is a merged PR that bumps `Cargo.toml`'s version. Don't tag by hand:
`tag-release.yml` tags the merge commit `vX.Y.Z`. That tag publishes to crates.io,
and `dist` (cargo-dist) builds the binaries and GitHub Release, regenerates
`Formula/skref.rb` in `alephic-ai/homebrew-tap` (`homebrew.yml`), and moves the `v1`
tag (`move-major-tag.yml`). The `v` prefix matters: `action.yml` downloads
`releases/download/v$VERSION/…`.

See [`doc/releasing.md`](doc/releasing.md) for the details and how to recover from a
failed run.

## Fidelity notes

- The frontmatter split mirrors Python's `content.split("---", 2)`.
- Names are NFKC-normalized before validation and directory-name comparison.
- `html.escape(quote=True)` is reproduced in `prompt::html_escape` (escapes `& < > " '`).
- `read-properties` JSON uses 2-space indent and preserves insertion order to match `json.dumps(indent=2)`.
- `--allow-claude-fields` (CLI flag, library `allow_claude_fields` bool parameter, action input `allow-claude-fields`) is a `skref` extension beyond the Python `skills-ref`: it whitelists Claude Code's `CLAUDE_FIELDS` for `validate` and surfaces them in `read-properties`. Default behavior is unchanged, so base-spec fidelity holds when the flag is off.
