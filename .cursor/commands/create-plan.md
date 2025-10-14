# Create Implementation Plan

## Instructions for Cursor Agent

When creating implementation plans in Plan mode, please follow these key requirements:

### 1. Language
- **Write all plan content in English**

### 2. Reference Past Plans
- **Use existing plans in `.cursor/plans/` as templates**
- Follow the same comprehensive implementation approach

### 3. Include Feedback Section for Rules Update
- **Always include "Implementation Feedback for Rules Update" section**
- This section should capture recurring issues to prevent future mistakes:
  - Common Clippy warnings that require repeated fixes (e.g., `uninlined_format_args`, type conversion issues)
  - Debugging time-consuming issues that could be prevented
  - Implementation patterns that consistently cause problems
  - Error handling approaches that need standardization
- **CRITICAL**: Only document issues with significant impact that would benefit future implementations
- **Avoid rule bloat**: If no major recurring issues were encountered, explicitly state "No significant issues requiring rule updates"
- LLMs tend to be verbose - resist the urge to add unnecessary rules for minor issues
- Add a task to update `.cursor/rules/implementation-rules.mdc` only when substantial improvements are identified

### 4. Post-Implementation Tasks
- **Always include task to update plan's To-dos status after implementation completion**
- Update the To-dos section at the end of the implementation plan in `.cursor/plans/`
- Mark completed tasks as "completed" and update any remaining tasks with current status
- This ensures plan documents reflect the actual implementation progress

- **Always include task to update plan content based on final implementation**
- Review the actual implementation against the original plan
- Update plan sections to reflect what was actually implemented:
  - Technical approach details that changed during implementation
  - File structure and module organization that evolved
  - Implementation patterns and decisions that differed from the plan
  - Any scope changes or feature additions/removals
- This prevents plan documents from becoming outdated and ensures they remain valuable references for future implementations

### 5. Implementation Completion Workflow
- **Always include tasks for post-implementation verification and submission**
- Follow `.cursor/commands/run-all-tests.md` to verify all check items pass
- Follow `.cursor/commands/commit.md` to create proper commits
- Follow `.cursor/commands/create-pr.md` to create pull requests
