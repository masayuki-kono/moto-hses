# Create Pull Request

## Overview

This document outlines the comprehensive procedures for creating and managing pull requests in the Moto-HSES project. It covers title formatting, description templates, and CLI-based PR creation to ensure consistent, high-quality pull requests that follow project standards and facilitate effective code review processes.

## Procedures

### Title Format

Follow conventional commit format:

```
type: brief description
```

Examples:

- `feat: implement comprehensive file operations`
- `fix: resolve file handling issues`
- `refactor: improve mock server architecture`

### Description Template

**REQUIRED**: Use the structured format defined in `.github/pull_request_template.md` for all pull requests.

When creating PRs, you MUST:
1. **Read the template**: Reference `.github/pull_request_template.md` content
2. **Follow the structure**: Use the template's sections (Overview, Major Changes, Technical Details, etc.)
3. **Fill all sections**: Complete each required section with relevant information
4. **Use emojis**: Include appropriate emojis (✨, 🔧, 🐛, 🏗️) for change types
5. **Reference issues**: Link related issues using "Closes #" format

**Template Structure**:
- Overview: Brief description of what the PR accomplishes
- Major Changes: Categorized list with emojis
- Technical Details: Implementation specifics
- Breaking Changes: List any breaking changes
- Related Issues: Link to issues using "Closes #"

**AVOID in PR descriptions**:
- ❌ **File Changes**: Do not include detailed file change lists
- ❌ **Code blocks with file paths**: Avoid `moto-hses-proto/src/` style listings
- ❌ **Command-like syntax**: Avoid text that could be interpreted as shell commands
- ❌ **Excessive technical details**: Keep descriptions concise and high-level
- ❌ **Raw git output**: Do not include git status or diff information

**Keep PR descriptions**:
- ✅ **High-level overview**: Focus on what the PR accomplishes
- ✅ **Business impact**: Explain why the changes matter
- ✅ **User-facing changes**: Describe visible improvements
- ✅ **Concise technical details**: Only essential implementation notes

### CLI PR Creation

When creating PRs via CLI, use the following command structure:

```bash
gh pr create --title "type: brief description" --body "PR description" --assignee @me
```

**Key points**:
- Use `--assignee @me` to automatically assign the PR to yourself
- This ensures proper ownership and review workflow
