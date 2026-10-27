#!/usr/bin/env bash
# pr-via-bipr — PreToolUse hook: a new PR in this repo opens only through the ptbr-writer
# `bipr` skill. Blocks `gh pr create` and the GitHub MCP create-PR tool unless the PR body
# carries bipr's footer. `gh pr edit` and every other command pass.
set -euo pipefail

FOOTER_MARK='[`bipr`](https://github.com/h0ffmann/ptbr-writer'

# hook JSON on stdin -> "<tool_name>\t<command>\t<body>" (empty fields when absent)
read_input() {
  python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
i=d.get("tool_input") or {}
print(d.get("tool_name",""), i.get("command",""), i.get("body",""), sep="\x1f", end="")' 2>/dev/null || true
}

bipr_skill_path() {
  ls -d "$HOME"/.claude/plugins/cache/ptbr-writer/ptbr-writer/*/skills/bipr/SKILL.md 2>/dev/null | sort -V | tail -1
}

# body_file_of <command>: the literal path after --body-file / -F, or empty
body_file_of() {
  python3 -c 'import shlex,sys
try: a=shlex.split(sys.argv[1])
except ValueError: sys.exit(0)
for k,t in enumerate(a):
  if t in ("--body-file","-F") and k+1<len(a): print(a[k+1]); break
  if t.startswith("--body-file="): print(t.split("=",1)[1]); break' "$1" 2>/dev/null || true
}

deny() {
  local skill; skill="$(bipr_skill_path)"
  local reason="New PRs in this repo open through the ptbr-writer bipr skill. Follow ${skill:-the bipr SKILL.md of the ptbr-writer plugin} step by step, or ask the user to type /bipr. Write the body to a file with a literal path and pass it with --body-file; the body must end with bipr's footer.md."
  python3 -c 'import json,sys
print(json.dumps({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":sys.argv[1]}}))' "$reason"
  exit 0
}

IFS=$'\x1f' read -r -d '' tool command body < <(read_input; printf '\0') || true

case "$tool" in
  Bash)
    printf '%s' "$command" | grep -Eq '(^|[;&|[:space:]])gh[[:space:]]+pr[[:space:]]+(create|new)([[:space:]]|$)' || exit 0
    file="$(body_file_of "$command")"
    [ -n "$file" ] && [ -f "$file" ] && grep -qF "$FOOTER_MARK" "$file" && exit 0
    deny
    ;;
  mcp__github__create_pull_request)
    printf '%s' "$body" | grep -qF "$FOOTER_MARK" && exit 0
    deny
    ;;
esac
exit 0
