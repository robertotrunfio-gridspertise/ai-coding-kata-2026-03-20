# Commit and Create PR

Commit all staged/unstaged changes and open a pull request with a generated description.

Optional argument: a short hint about what changed (e.g. "refactor go kata"). If omitted, infer from the diff.

## Steps

### 1. Understand the current state

Run these in parallel:
```bash
git status
git diff HEAD
git log --oneline -10
```

If there is nothing to commit (clean working tree and no staged changes), stop and tell the user.

### 2. Run tests before committing

Run all three test suites to confirm nothing is broken. Use the steps from the `/test all` skill (check toolchains, install if missing, run tests). If any test suite fails, stop, report the failures, and do NOT commit until the user confirms they want to proceed anyway.

### 3. Stage changes

Stage only source files — never stage secrets, build artifacts, or binaries:
```bash
git add go-kata/ java-kata/ rust-kata/ prompts/ solution.md .claude/
```

Exclude: `*.class`, `build/`, `target/`, `.env`, `*.log`.

### 4. Write the commit message

Analyse `git diff --staged` and write a message that:
- Has a short subject line (≤ 72 chars) summarising **what** changed
- Has a blank line, then a body that explains **why** and lists the key changes as bullet points
- Ends with: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`

Commit using a heredoc to preserve formatting:
```bash
git commit -m "$(cat <<'EOF'
<subject line>

<body>

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
EOF
)"
```

### 5. Push to remote

Check whether the current branch already has a remote tracking branch:
```bash
git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null
```

If it does, push normally:
```bash
git push
```

If it does not, push and set upstream:
```bash
git push -u origin HEAD
```

### 6. Create the pull request

Use `gh pr create` with a body that includes:

- **Summary** — 2–4 bullet points describing what changed and why
- **Changes by language** — one section per language that was touched, listing files modified/created
- **Test results** — how many tests pass per language
- **Notes** — anything worth calling out (rejected design, new skill files, etc.)

```bash
gh pr create --title "<short title>" --body "$(cat <<'EOF'
## Summary
- ...

## Changes by language

### Go
- ...

### Java
- ...

### Rust
- ...

## Test results
| Language | Tests | Result |
|----------|-------|--------|
| Go       | 30/30 | PASS   |
| Java     | 28/28 | PASS   |
| Rust     | 27/27 | PASS   |

## Notes
- ...

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

After the PR is created, print the PR URL so the user can open it directly.
