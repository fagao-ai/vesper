# Release Guide

This document describes how to create releases for Vesper SSH Tunnel Manager.

## GitHub CI Workflows

We have two GitHub Actions workflows set up for automated building and testing:

### 1. `.github/workflows/build-and-release.yml`

This is the main workflow for building and releasing the application:

**Trigger Conditions:**
- Push tags with `v` prefix (e.g., `v1.0.0`)
- Manual trigger (workflow_dispatch)

**Supported Platforms:**
- macOS (macos-latest)
- Windows (windows-latest)
- Linux (ubuntu-latest)

**Main Features:**
- Automatically installs required dependencies for each platform
- Sets up Rust and Node.js environments
- Installs frontend dependencies using pnpm
- Builds and packages the Tauri application
- Automatically creates GitHub Release and uploads build artifacts

### 2. `.github/workflows/pr-check.yml`

This is the Pull Request check workflow:

**Trigger Conditions:**
- Created or updated Pull Requests

**Check Items:**
- TypeScript type checking
- Frontend build testing
- Rust code checking
- Running test suite

## How to Use

### 1. Release a New Version

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

Pushing a tag will automatically trigger the build process.

### 2. Build Artifacts

After the build is complete, artifacts are automatically uploaded to GitHub Releases, including:
- **macOS**: `.dmg` file
- **Windows**: `.msi` installer
- **Linux**: `.AppImage` or `.deb` package

### 3. Pull Request Checks

Every PR automatically runs checks to ensure the code builds correctly on all platforms.

## Notes

- The workflow is optimized for your project configuration (using pnpm, Tauri v2)
- Releases are initially created as drafts and need to be manually published
- All build artifacts are signed and notarized where applicable
- The build process may take 10-20 minutes depending on the platform