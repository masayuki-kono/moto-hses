# CI/CD Setup

This project uses GitHub Actions to build a CI/CD pipeline.

## Workflows

### 1. Rust Tests (`rust-tests.yml`)

Automatically runs on pull requests or pushes to the main branch.

**Execution contents:**

- Code formatting check (`cargo fmt`)
- Static analysis with Clippy (`cargo clippy`)
- Unit test execution (`cargo test`)
- Test execution in release mode

### 2. Security Audit (`security-audit.yml`)

Automatically runs on pull requests to main branch and every Sunday.

**Execution contents:**

- Security audit (`cargo audit`)
- Dependency update check (`cargo outdated`)
- Unused dependency check (`cargo machete`)

## Branch Protection Rules Setup

It is recommended to set up the following branch protection rules in GitHub repository settings:

### Main Branch Protection

1. **Require a pull request before merging**

   - ✅ Enable
   - ✅ Require approvals: 1 or more
   - ✅ Dismiss stale PR approvals when new commits are pushed

2. **Require status checks to pass before merging**

   - ✅ Enable
   - Required checks:
     - `Rust Tests / Run Tests`
     - `Security Audit / Security Checks`

3. **Require branches to be up to date before merging**

   - ✅ Enable

4. **Require conversation resolution before merging**
   - ✅ Enable

## Local Pre-check

Before creating a pull request, run the following commands for local checks:

```bash
# Format check
cargo fmt --all -- --check

# Clippy check
cargo clippy --all-targets --all-features -- -D warnings

# Test execution
cargo test --all-features --workspace

# Security audit
cargo audit

# Dependency check
cargo outdated
```

## Troubleshooting

### Common Issues

1. **Format errors**

   ```bash
   cargo fmt --all
   ```

2. **Clippy warnings**

   ```bash
   cargo clippy --fix --all-targets --all-features
   ```

3. **Test failures**
   - Check test cases
   - Check dependencies
   - Check environment variable settings

### Support

If issues cannot be resolved, please report them in GitHub Issues.
