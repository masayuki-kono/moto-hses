# Run Background Agent

## Overview

This command starts a Background Agent to execute implementation tasks based on a specified branch and implementation plan. The agent will follow the plan systematically, executing each task and updating progress as it works through the implementation.

## Usage

```bash
# Start background agent with branch and plan
/run-background-agent <branch-name> <plan-filename>

# Example
/run-background-agent feature/add-hses-parser 0x301-plural-register-command-663489d7.plan.md
```

## Parameters

- **branch-name**: The git branch to work on (e.g., `feature/add-hses-parser`, `fix/resolve-connection-timeout`)
- **plan-filename**: Filename of the implementation plan in `.cursor/plans/` directory (e.g., `0x301-plural-register-command-663489d7.plan.md`)

## Workflow

The Background Agent will:

1. **Switch to specified branch**
   - Checkout the target branch
   - Verify branch exists and is clean

2. **Load implementation plan**
   - Read the plan file from `.cursor/plans/<plan-filename>`
   - Parse tasks and requirements
   - Understand implementation scope

3. **Execute implementation tasks**
   - Work through each task systematically
   - Follow implementation rules from `.cursor/rules/implementation-rules.mdc`
   - Run tests after each step to ensure quality

4. **Update progress**
   - Mark completed tasks in the plan
   - Update implementation status
   - Document any issues or deviations

5. **Follow completion workflow**
   - Run all tests using `.cursor/commands/run-all-tests.md`
   - Update plan content based on final implementation
   - Create commits using `.cursor/commands/commit.md`
   - Create pull request using `.cursor/commands/create-pr.md`

## Prerequisites

- Valid git branch must exist
- Implementation plan file must be accessible
- All required dependencies must be installed
- Development environment must be properly configured

## Error Handling

The agent will:
- Stop execution if branch doesn't exist
- Report errors if plan file is not found

