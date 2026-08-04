@AGENTS.md

## Claude Code-specific notes

- No project-specific skills or slash commands are configured for this
  repository as of this writing.
- The user has previously asked for autonomous, multi-step work (implement
  → verify → publish → push) without pausing for confirmation at each
  step, so long as the actions stay within the scope of what was asked
  (crate implementation, publishing to crates.io, pushing to the three
  configured git remotes). Destructive or scope-expanding actions still
  warrant a check-in — see the general operating instructions.
- `gh` (GitHub CLI) is authenticated and available for checking CI status
  after a push — use it; don't assume a push succeeded CI just because it
  built locally (see `AGENTS/lessons.md`).
