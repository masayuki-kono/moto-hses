# Run Background Agent

## Overview

This command starts a Background Agent to execute implementation tasks based on a specified implementation plan. The agent will follow the plan systematically, executing each task and updating progress as it works through the implementation.

## Usage

```bash
# Start background agent with plan
/run-background-agent <plan-filename>

# Example
/run-background-agent 0x301-plural-register-command-663489d7.plan.md
```

## Parameters

- **plan-filename**: Filename of the implementation plan in `.cursor/plans/` directory (e.g., `0x301-plural-register-command-663489d7.plan.md`)

## Workflow

The Background Agent will:

1. **Load implementation plan**
   - Read the plan file from `.cursor/plans/<plan-filename>`
   - Parse tasks and requirements
   - Understand implementation scope

2. **Execute implementation tasks**
   - Work through each task systematically
   - Follow implementation rules from `.cursor/rules/implementation-rules.mdc`
   - Run tests after each step to ensure quality

## Prerequisites

- Implementation plan file must be accessible
