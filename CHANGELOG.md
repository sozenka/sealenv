# Changelog

All notable changes to sealenv will be documented here.

Format: [Semantic Versioning](https://semver.org)

## [Unreleased]

## [0.1.0] - 2025-XX-XX

### Added
- `sealenv init` — initialize project, generate key, auto-update .gitignore
- `sealenv encrypt` — encrypt .env → .env.enc with AES-256-GCM
- `sealenv decrypt` — decrypt .env.enc → .env
- `sealenv add KEY=VALUE` — add a secret to the current profile
- `sealenv profile create/use/list` — manage multiple env profiles
- `sealenv run -- <cmd>` — inject env vars into subprocess without writing .env to disk
- `sealenv diff <a> <b>` — show differences between two profiles
- `sealenv key export/import` — share keys securely with teammates
- Cross-compiled binaries for Linux, macOS (Intel + ARM), Windows
