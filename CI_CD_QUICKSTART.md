# CI/CD Quick Start Guide

## 🚀 What You Just Got

Your Recommendation Engine now has **enterprise-grade CI/CD automation** with 5 comprehensive workflows:

1. **✅ Test Workflow** - Automated testing on every PR
2. **🐳 Docker Workflow** - Automated container builds and security scanning
3. **🔒 Security Workflow** - Daily vulnerability scanning
4. **📊 Coverage Workflow** - Code coverage tracking
5. **🎉 Release Workflow** - Automated multi-platform releases

---

## 📋 Quick Checklist

### Immediate (Required)
- [ ] Commit and push the workflows to GitHub
- [ ] Verify first workflow run succeeds
- [ ] Set up branch protection rules

### Optional (Enhanced Features)
- [ ] Add `CODECOV_TOKEN` secret for coverage integration
- [ ] Configure Dependabot for automated updates
- [ ] Add status badges to README

---

## 🎯 Committing the Workflows

```bash
# Check what we created
git status

# Add all CI/CD files
git add .github/workflows/
git add deny.toml
git add TASK_28_CI_CD_IMPLEMENTATION.md
git add CI_CD_QUICKSTART.md

# Commit
git commit -m "feat: Add comprehensive CI/CD pipelines

- Add GitHub Actions workflows for testing, security, coverage
- Add Docker build and multi-platform release automation
- Add cargo-deny configuration for license compliance
- Include daily security audits with cargo-audit
- Add code coverage reporting with llvm-cov

Implements Task 28 (CI/CD Pipeline)"

# Push to trigger first run
git push origin main
```

---

## 👀 Watching Your First Run

1. **Go to GitHub Actions tab**
   ```
   https://github.com/YOUR_ORG/recommendation-engine/actions
   ```

2. **You'll see workflows running:**
   - Test (first to complete)
   - Security Audit
   - Code Coverage
   - Docker Build

3. **Expected results:**
   - ✅ Test: ~5-8 minutes
   - ✅ Security: ~8-12 minutes
   - ✅ Coverage: ~6-10 minutes
   - ✅ Docker: ~10-15 minutes

---

## 🛡️ Setting Up Branch Protection

### Navigate to Settings:
```
Repository → Settings → Branches → Add rule
```

### Recommended Configuration:

**Branch name pattern:** `main`

**Protect matching branches:**
- ✅ Require a pull request before merging
  - ✅ Require approvals: 1
  - ✅ Dismiss stale pull request approvals
  - ✅ Require review from Code Owners (optional)

- ✅ Require status checks to pass before merging
  - ✅ Require branches to be up to date
  - **Required checks:**
    - Test Suite
    - Integration Tests
    - Dependency Audit
    - Cargo Deny Check
    - Build Docker Image

- ✅ Require conversation resolution before merging
- ✅ Do not allow bypassing the above settings
- ❌ Allow force pushes
- ❌ Allow deletions

---

## 📈 Adding Status Badges to README

Add these to the top of your `README.md`:

```markdown
# Recommendation Engine

[![Test](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/test.yml/badge.svg)](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/test.yml)
[![Security Audit](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/security.yml/badge.svg)](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/security.yml)
[![Docker](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/docker.yml/badge.svg)](https://github.com/YOUR_ORG/recommendation-engine/actions/workflows/docker.yml)
[![codecov](https://codecov.io/gh/YOUR_ORG/recommendation-engine/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_ORG/recommendation-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, domain-agnostic recommendation microservice written in Rust.
```

---

## 🔐 Optional: Codecov Integration

### 1. Sign up at codecov.io
```
https://codecov.io
```

### 2. Add repository
- Link your GitHub account
- Select the recommendation-engine repo

### 3. Get token
- Copy the upload token

### 4. Add to GitHub Secrets
```
Repository → Settings → Secrets → Actions → New secret
Name: CODECOV_TOKEN
Value: [paste token]
```

### 5. Push again
Coverage will now be uploaded and tracked!

---

## 🚢 Creating Your First Release

### 1. Update Version in Code
```bash
# Cargo.toml already has version 1.0.0
# No changes needed for first release
```

### 2. Create and Push Tag
```bash
# Create annotated tag
git tag -a v1.0.0 -m "Release v1.0.0

- Complete recommendation engine implementation
- Full CI/CD automation
- Production-ready features"

# Push tag to trigger release
git push origin v1.0.0
```

### 3. Watch Release Workflow
- Builds binaries for 5 platforms
- Publishes Docker images
- Creates GitHub Release with changelog
- **Duration:** ~25-35 minutes

### 4. Download Artifacts
```
GitHub → Releases → v1.0.0
```

**Available:**
- recommendation-api-linux-amd64.tar.gz
- recommendation-api-linux-amd64-musl.tar.gz
- recommendation-api-macos-amd64.tar.gz
- recommendation-api-macos-arm64.tar.gz
- recommendation-api-windows-amd64.exe.zip

**Docker:**
```bash
docker pull ghcr.io/YOUR_ORG/recommendation-engine:1.0.0
docker pull ghcr.io/YOUR_ORG/recommendation-engine:latest
```

---

## 🧪 Testing Workflows Locally

### Before Pushing (Syntax Check)
```bash
# Install yamllint
pip install yamllint

# Check workflow syntax
yamllint .github/workflows/*.yml
```

### Running Tests Locally (Match CI)
```bash
# Format check
cargo fmt --all -- --check

# Clippy with same strictness as CI
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --workspace --verbose

# Integration tests (with services running)
cargo test -p recommendation-integration-tests -- --test-threads=1
```

### Docker Build Test
```bash
# Build exactly as CI does
docker build -t recommendation-engine:test .

# Test container
docker run -d --name test \
  --network host \
  -e DATABASE_URL=postgresql://postgres:postgres@localhost:5432/recommendations \
  -e REDIS_URL=redis://localhost:6379 \
  -e API_KEY=test-key \
  recommendation-engine:test

# Verify
curl http://localhost:8080/health
curl http://localhost:8080/ready

# Cleanup
docker stop test && docker rm test
```

---

## 🐛 Troubleshooting

### Workflow Fails on First Run

**Test failures:**
- Check that PostgreSQL/Redis services started
- Look for "Service Unhealthy" in logs
- Verify DATABASE_URL and REDIS_URL in workflow

**Docker build timeout:**
- First build takes longer (no cache)
- Subsequent builds: ~5 minutes
- Patience on first run!

**Security audit fails:**
- Check `deny.toml` configuration
- Review failed advisories
- Add exceptions if false positives

### PR Checks Not Required

**Enable branch protection:**
1. Settings → Branches
2. Add rule for `main`
3. Check "Require status checks"
4. Select the checks

### Coverage Not Uploading

**Missing CODECOV_TOKEN:**
- Workflow will succeed but not upload
- Coverage report still generated as artifact
- Add token to enable upload

---

## 📊 Monitoring CI/CD Health

### Daily Checks:
- ✅ Green checkmarks on recent commits
- ✅ No security advisories reported

### Weekly Checks:
- ✅ Review failed workflows (if any)
- ✅ Update outdated dependencies
- ✅ Check coverage trends

### Monthly Checks:
- ✅ Update workflow actions versions
- ✅ Review and optimize cache strategy
- ✅ Audit performance (build times)

---

## 🎓 Understanding Workflow Triggers

### On Every Push to main/develop:
- ✅ Test Workflow
- ✅ Security Workflow
- ✅ Coverage Workflow
- ✅ Docker Build (main only)

### On Every Pull Request:
- ✅ Test Workflow
- ✅ Security Workflow
- ✅ Coverage Workflow
- ✅ Docker Build (test only, not published)

### Daily (2 AM UTC):
- ✅ Security Workflow (scheduled)

### On Version Tag (v*):
- ✅ Release Workflow (only)

---

## 🚀 What Happens Next

### Every Pull Request:
1. Developer creates PR
2. All checks run automatically
3. Status checks appear on PR
4. Coverage report posted as comment
5. Team reviews code + checks
6. Merge when green ✅

### Every Merge to Main:
1. All workflows run
2. Docker image built and published
3. Coverage tracked over time
4. Security audit completed
5. Latest image available at `ghcr.io/.../recommendation-engine:main`

### Every Release Tag:
1. Release created with changelog
2. Binaries built for 5 platforms
3. Docker images tagged with version
4. Artifacts attached to release
5. Team notified ✨

---

## 💡 Pro Tips

### Speed Up CI:
```yaml
# Add path filters to workflows
on:
  push:
    paths-ignore:
      - '**.md'
      - 'docs/**'
```

### Parallel Testing:
```bash
# CI runs tests in parallel
cargo test --workspace
```

### Cache Hits:
- First run: ~15 minutes
- Cached runs: ~5 minutes
- Savings: 66%! 🎉

### Security:
- Runs daily even without code changes
- Catches new vulnerabilities
- Zero-day protection

---

## 🎯 Success Criteria

After committing and pushing, you should see:

- ✅ All workflows appear in Actions tab
- ✅ First runs complete successfully
- ✅ Green checkmarks on commits
- ✅ Docker image in Container Registry
- ✅ Coverage report generated
- ✅ No security advisories

---

## 📚 Additional Resources

### GitHub Actions Docs:
- https://docs.github.com/en/actions

### Cargo Security Tools:
- cargo-audit: https://github.com/rustsec/rustsec
- cargo-deny: https://embarkstudios.github.io/cargo-deny/

### Coverage Tools:
- llvm-cov: https://github.com/taiki-e/cargo-llvm-cov
- Codecov: https://about.codecov.io/

---

## ✨ You're Ready!

Your recommendation engine now has:
- ✅ Automated testing
- ✅ Security scanning
- ✅ Code coverage
- ✅ Docker automation
- ✅ Multi-platform releases

**Time to push and watch the magic happen!** 🚀

```bash
git add -A
git commit -m "feat: Add CI/CD pipelines"
git push origin main
```

Then visit: `https://github.com/YOUR_ORG/recommendation-engine/actions`

---

## Need Help?

See `TASK_28_CI_CD_IMPLEMENTATION.md` for detailed documentation on each workflow, troubleshooting, and advanced configuration.

Happy shipping! 🎉
