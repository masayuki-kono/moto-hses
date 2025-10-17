# Prepare Crate Release

## Overview

This document outlines the procedures for preparing crate release in the Moto-HSES project.

## Release Preparation Tasks

When preparing a new release, the following tasks must be completed. The version number (e.g., "0.2.0") should be provided as an argument.

### Prerequisites
- Version number must be specified (e.g., "0.2.0")
- All development changes should be committed and ready for release

### 1. Update Version Numbers
- Update `version` field in all `Cargo.toml` files (moto-hses-client, moto-hses-proto, moto-hses-mock)
- Update dependency version references in `Cargo.toml` files (e.g., `moto-hses-proto = { version = "0.2" }`)

### 2. Update README.md Files
- Update version numbers in usage examples in all README.md files
- Replace version strings like `moto-hses-client = "0.1"` with new version

### 3. Update CHANGELOG.md Files
- Add new release entry with current date
- Include list of newly added commands by comparing current README with previous tag's README
- Format: `## [VERSION] - DATE` followed by `### Features` section with command list

### 4. Update Cargo.lock
- Run `cargo update` to regenerate Cargo.lock with new version numbers

### 5. Security and Quality Checks
- Run `cargo audit` to check for security vulnerabilities
- Run `cargo outdated --exit-code 1` to check for outdated dependencies
- Run `cargo machete` to check for unused dependencies
- If any issues are found, resolve them before proceeding with release

## Command Detection Process

To identify newly added commands:
1. Get the latest git tag: `git tag --sort=-version:refname | head -1`
2. Compare current README.md with previous tag's README.md
3. Extract command table entries that exist in current but not in previous version
4. Format as bullet points in CHANGELOG entry

## Example CHANGELOG Entry Format

```markdown
## [0.2.0] - 2025-01-17

### Features
- **Robot Control Commands**:
  - Start-up (Job Start) (0x86)
  - Job Select (0x87)
  - Plural I/O Data Reading/Writing (0x300)
  - Plural Register Data Reading/Writing (0x301)
  - Plural Variable Operations (0x302-0x306): Byte, Integer, Double Precision Integer, Real, Character types
```

## Files Modified During Release Preparation

The following files are typically modified during release preparation:

- `Cargo.lock` - Updated with new version numbers
- `moto-hses-client/Cargo.toml` - Version and dependency updates
- `moto-hses-client/README.md` - Version number in usage examples
- `moto-hses-client/CHANGELOG.md` - New release entry with added commands
- `moto-hses-mock/Cargo.toml` - Version and dependency updates
- `moto-hses-mock/README.md` - Version number in usage examples
- `moto-hses-mock/CHANGELOG.md` - New release entry with added commands
- `moto-hses-proto/Cargo.toml` - Version updates
- `moto-hses-proto/README.md` - Version number in usage examples
- `moto-hses-proto/CHANGELOG.md` - New release entry with added commands

## Key Points

- All crates must be updated consistently with the same version number provided as argument
- CHANGELOG entries should include the current date and list of newly added commands
- New commands are identified by comparing current README with previous tag's README
- Dependency versions in Cargo.toml files must be updated to match the new version
- Security and quality checks must pass before release (cargo audit, cargo outdated, cargo machete)
- If security issues are found, they must be resolved before proceeding
- After preparation, review changes with `git diff --cached` before committing
