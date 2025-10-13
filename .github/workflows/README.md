# GitHub Actions Workflows

This directory contains GitHub Actions CI/CD workflows for EnvMesh.

## Workflows

### `ci.yml` - Continuous Integration

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`

**What it does:**
1. **Check Job** - Runs on Linux, macOS, Windows
   - Compiles code (`cargo check`)
   - Checks formatting (`cargo fmt`)
   - Runs linter (`cargo clippy`)
   - Runs tests (`cargo test`)

2. **Build Job** - Builds release binaries
   - Compiles optimized binaries
   - Uploads artifacts for download

**Runtime:** ~10-15 minutes

**Status Badge:**
```markdown
![CI](https://github.com/YOUR_USERNAME/envmesh/workflows/CI/badge.svg)
```

---

### `release.yml` - Release Automation

**Triggers:**
- Git tags matching `v*.*.*` (e.g., `v0.1.0`)

**What it does:**
1. Creates GitHub Release
2. Builds binaries for all platforms:
   - Linux (x86_64)
   - macOS (x86_64 + ARM64/M1)
   - Windows (x86_64)
3. Uploads binaries as release assets

**Runtime:** ~20-30 minutes

**How to use:**
```bash
# Create and push a tag
git tag v0.1.0
git push origin v0.1.0

# GitHub Actions will automatically:
# 1. Build binaries for all platforms
# 2. Create a release
# 3. Upload binaries
```

---

## Setup Instructions

### First Time Setup

1. **Push workflows to GitHub:**
   ```bash
   git add .github/workflows/
   git commit -m "Add GitHub Actions CI/CD"
   git push
   ```

2. **Verify workflows:**
   - Go to your repo on GitHub
   - Click "Actions" tab
   - You should see workflows listed

3. **Enable Actions (if needed):**
   - Go to Settings → Actions → General
   - Enable "Allow all actions and reusable workflows"

### Testing CI

**Trigger a CI run:**
```bash
# Make any change and push
git commit --allow-empty -m "Test CI"
git push
```

**View results:**
- Go to Actions tab on GitHub
- Click on the workflow run
- See results for each platform

### Creating a Release

**Step 1: Tag your code**
```bash
# Update version in Cargo.toml first
git add src-tauri/Cargo.toml
git commit -m "Bump version to 0.1.0"

# Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

**Step 2: Wait for build**
- Go to Actions tab
- Watch "Release" workflow
- Wait ~20-30 minutes

**Step 3: Check releases**
- Go to Releases tab
- Your release should be there with binaries

---

## Caching

Both workflows use cargo caching to speed up builds:
- Cargo registry cache
- Cargo git cache
- Build artifact cache

**First run:** ~15-20 minutes
**Cached runs:** ~5-10 minutes

---

## Troubleshooting

### CI Fails on Formatting

**Error:** `cargo fmt -- --check` fails

**Fix:**
```bash
# Format code locally
cd src-tauri
cargo fmt

# Commit and push
git add .
git commit -m "Format code"
git push
```

### CI Fails on Clippy

**Error:** Clippy warnings treated as errors

**Fix:**
```bash
# Run clippy locally
cd src-tauri
cargo clippy --all-targets --all-features -- -D warnings

# Fix warnings
# ...

# Commit and push
git add .
git commit -m "Fix clippy warnings"
git push
```

### Release Workflow Doesn't Trigger

**Possible causes:**
1. Tag not pushed: `git push origin v0.1.0`
2. Tag format wrong: Must be `v*.*.*` (e.g., `v0.1.0`)
3. Actions disabled in repo settings

**Check:**
```bash
# List tags
git tag

# Check remote tags
git ls-remote --tags origin

# Ensure Actions are enabled
# Go to Settings → Actions → General
```

### Build Fails on Platform

**Error:** Build fails on specific OS

**Debug:**
1. Check workflow logs in Actions tab
2. Look for platform-specific errors
3. Test locally if possible
4. Check system dependencies (Linux only)

---

## Local Testing (Optional)

You can test workflows locally using [act](https://github.com/nektos/act):

### Install act

```bash
# macOS
brew install act

# Linux
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

### Run CI locally

```bash
# Run all jobs
act -W .github/workflows/ci.yml

# Run specific job
act -W .github/workflows/ci.yml -j check

# Run on specific platform
act -W .github/workflows/ci.yml -j check -P ubuntu-latest=catthehacker/ubuntu:full-latest
```

**Note:** Local testing has limitations:
- No macOS/Windows on Linux host
- Some GitHub Actions features not supported
- Useful for quick validation only

---

## Future Improvements

### Potential additions:

1. **Security Audit**
   ```yaml
   - name: Security audit
     run: cargo audit
   ```

2. **Code Coverage**
   - Use tarpaulin or grcov
   - Upload to Codecov

3. **Benchmarks**
   - Run criterion benchmarks
   - Track performance over time

4. **Documentation**
   - Build and deploy docs
   - rustdoc to GitHub Pages

5. **Dependency Updates**
   - Enable Dependabot
   - Auto-update dependencies

---

## Status Badges

Add to your README.md:

```markdown
[![CI](https://github.com/YOUR_USERNAME/envmesh/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/envmesh/actions/workflows/ci.yml)
[![Release](https://github.com/YOUR_USERNAME/envmesh/workflows/Release/badge.svg)](https://github.com/YOUR_USERNAME/envmesh/actions/workflows/release.yml)
```

---

## Cost

**Free for public repos:**
- 2,000 minutes/month for free
- EnvMesh CI: ~10 min/run = ~200 runs/month
- Plenty for development

**For private repos:**
- 3,000 minutes/month (Pro plan)
- Or use self-hosted runners
