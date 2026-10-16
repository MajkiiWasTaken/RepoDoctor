# RepoDoctor

A lightweight cross-platform CLI for Git repository diagnostics, project checks, and release readiness, written in Rust.

**Platforms:** Windows / Linux

## Features

- Git status, branch, remotes, and upstream checks
- Rust, .NET, and Node.js project discovery
- Tracked build artifact and large-file detection
- Release readiness checks with strict/CI modes
- TXT and JSON reports; optional `repodoctor.toml` configuration

## Installation

Download a package from [GitHub Releases](https://github.com/MajkiiWasTaken/RepoDoctor/releases).

**Windows:** Extract the ZIP and run:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

**Linux:** Extract the archive and run:

```bash
bash ./install.sh
```

Or build from source with `cargo build --release`.

## Usage

```bash
rdoctor scan                          # Inspect current project
rdoctor scan /path/to/project         # Inspect another project
rdoctor git                           # Check Git repository
rdoctor release                       # Check release readiness
rdoctor release --strict              # Treat warnings as failures
rdoctor release --ci --export report.json --format json
```

## Configuration

Optionally create `repodoctor.toml` in the project root:

```toml
[files]
large_file_mb = 20

[release]
require_license = false
warn_existing_tag = false
```

Reports are not overwritten if they already exist. Git upstream comparisons use local tracking references and do not run `git fetch`.

---

**Author:** Michal Švrček

**License:** MIT
