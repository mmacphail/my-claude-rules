---
name: malky-skill-creator
description: Creates a new personal Claude Code skill following malky best practices. Use when asked to create, scaffold, or add a new skill.
argument-hint: <skill-name> [purpose]
disable-model-invocation: true
allowed-tools: Bash, Write, Read, Glob
---

# Malky Skill Creator

Scaffold a new personal Claude Code skill for Alexandre's setup.

## Skill location

All personal skills live at: `~/.claude/skills/<skill-name>/SKILL.md`

## Steps

1. **Determine the skill name and purpose** from $ARGUMENTS (or ask if missing)
   - Name: lowercase, hyphens only, max 64 chars, gerund form preferred (`processing-pdfs`, `writing-commits`)
   - Purpose: what it does + when Claude should use it

2. **Decide the skill type** based on purpose:
   - Has side effects (writes files, runs deploys, sends messages) → `disable-model-invocation: true`
   - Pure background knowledge → `user-invocable: false`
   - Heavy research/exploration → `context: fork` + `agent: Explore`
   - Default: user-invocable, auto-invocable

3. **Create the skill directory and SKILL.md**

   ```bash
   mkdir -p ~/.claude/skills/<skill-name>
   ```

4. **Write SKILL.md** following these rules:
   - Description: third person, specific, includes triggers ("Use when...")
   - Body: under 500 lines, no padding, assume Claude is smart
   - Degrees of freedom: high (text) for flexible tasks, low (exact scripts) for fragile ones
   - If content is large: split into reference files linked one level deep from SKILL.md

5. **Create supporting files only if needed** (templates, reference docs, scripts)

6. **Confirm** what was created and show the final SKILL.md content

## SKILL.md template

```markdown
---
name: <skill-name>
description: <What it does. Use when <specific triggers>.>
argument-hint: <hint>        # omit if no args
disable-model-invocation: true  # omit if auto-invoke is fine
allowed-tools: Read, Bash   # only tools needed
---

# <Skill Title>

<Concise instructions. No fluff. Reference supporting files with [file.md](file.md) if needed.>
```

## Quality checklist before finishing

- [ ] Description is specific + includes "Use when..." triggers
- [ ] Description written in third person
- [ ] No vague names (`helper`, `utils`)
- [ ] Body is concise — no explaining things Claude already knows
- [ ] `disable-model-invocation: true` if the skill has side effects
- [ ] File references one level deep (no chains)
- [ ] Forward slashes in all paths
