---
description: Execute the full commit workflow with project plan updates and code quality checks
allowed-tools: Read, Edit, MultiEdit, Bash(cargo *), Bash(jj *)
---

Execute the mandatory pre-commit workflow in this exact order:

**Step 1: Update Project Plans FIRST**
- Check which story/subtask was just completed by examining recent changes
- Update progress in the relevant detailed epic plan (`project_plan_{epic}.md`)
- Mark completed stories/subtasks with ✅ and completion date
- Update acceptance criteria checkboxes from [ ] to [x]
- Update master plan (`project_plan.md`) epic status if needed
- Update technical_considerations.md with any lessons learned

**Step 2: Pre-Commit Code Quality**
- Run `cargo fmt` to format all code consistently
- Run `cargo check` to verify compilation without building

**Step 3: Create Commit**
- Run `jj commit -m "descriptive message"` with:
  - Story/subtask format: "Implement [feature] ([Story X.Y])" 
  - Always include Claude Code attribution in commit message

**Step 4: Post-Commit Lint Fixes**
- Run `cargo clippy --fix` to auto-fix linting issues (works after commit)
- Manually fix any remaining clippy warnings that don't have autofixes
- Run `cargo clippy --no-deps` for final lint check
- If any changes were made: run `jj squash` to squash fixes into parent commit

Execute all steps automatically without asking for confirmation.