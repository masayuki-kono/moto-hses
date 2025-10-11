# Create Branch

## Overview

This document outlines the procedures for creating branches in the Moto-HSES project. It covers branch naming conventions and creation commands based on feature requirements, implementation plans, or chat-specified content to ensure consistent and organized development workflow.

## Branch Naming Convention

- `feature/<short-description>`: New features
- `fix/<short-description>`: Bug fixes
- `refactor/<short-description>`: Code refactoring
- `docs/<short-description>`: Documentation updates
- `test/<short-description>`: Test-related changes
- `chore/<short-description>`: Maintenance or setup

> Use `-` instead of `_` for readability (e.g., `feature/add-hses-parser`).

## Branch Creation Examples

```bash
# Feature branch
git checkout -b feature/add-hses-parser

# Bug fix branch
git checkout -b fix/resolve-connection-timeout

# Refactoring branch
git checkout -b refactor/improve-mock-server

# Documentation branch
git checkout -b docs/update-api-documentation

# Test branch
git checkout -b test/add-integration-tests

# Maintenance branch
git checkout -b chore/update-dependencies
```

### From Implementation Plans

```bash
# Read implementation plan first
cat docs/implementation-plans/<plan-name>.md

# Create branch based on plan content
git checkout -b <type>/<plan-based-description>
```

### Auto-suggest Branch Name from Git Diff

When no specific branch name is provided, analyze git diff to suggest appropriate branch names:

```bash
# Check current changes
git status

# Analyze diff to suggest branch type and name
git diff --name-only
```

**Branch Name Suggestion Logic**:
- **Modified test files only** → `test/<test-description>`
- **Modified docs/ files only** → `docs/<doc-description>`
- **Modified Cargo.toml/Cargo.lock** → `chore/update-dependencies`
- **Modified src/ files** → `feature/<feature-description>` or `fix/<fix-description>`
- **Mixed changes** → `refactor/<refactor-description>`

**Example Auto-suggestion Process**:
```bash
# 1. Check what files are modified
git diff --name-only
# Output: src/commands/alarm.rs, tests/alarm_tests.rs

# 2. Suggest branch name based on changes
# → feature/improve-alarm-commands (if new functionality)
# → fix/alarm-command-issues (if bug fixes)
```
