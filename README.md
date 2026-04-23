# sealenv

**Encrypt .env files and switch profiles. One binary, zero setup.**

[![Crates.io](https://img.shields.io/crates/v/sealenv.svg)](https://crates.io/crates/sealenv)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![CI](https://github.com/sozenka/sealenv/actions/workflows/release.yml/badge.svg)](https://github.com/sozenka/sealenv/actions)

---

<!-- INSERT DEMO GIF HERE -->
<!-- Record with: vhs demo.tape -->

---

## The problem

Everyone leaks secrets eventually. `.env` files get committed. `.env.example` gets out of sync. Switching between dev/staging/prod means manually copying files. There's no standard, no tooling, and every team does it differently.

`sealenv` fixes this in one binary.

## Install

**Homebrew** *(coming soon)*
```sh
brew install sealenv
```

**Cargo**
```sh
cargo install sealenv
```

**Direct download**

Grab the binary for your platform from [Releases](https://github.com/sozenka/sealenv/releases).

```sh
# Linux / macOS
curl -L https://github.com/sozenka/sealenv/releases/latest/download/sealenv-linux-x86_64.tar.gz | tar xz
sudo mv sealenv /usr/local/bin/
```

---

## Quick start

```sh
# 1. Initialize sealenv in your project
sealenv init

# 2. Add your secrets
sealenv add DATABASE_URL=postgres://localhost/mydb
sealenv add API_KEY=sk-abc123

# 3. Encrypt .env — safe to commit
sealenv encrypt
git add .env.enc
git commit -m "chore: add encrypted env"

# 4. Run your app without ever writing .env to disk
sealenv run -- npm start
```

---

## Commands

| Command | Description |
|---|---|
| `sealenv init` | Initialize sealenv in a project, generate key, update .gitignore |
| `sealenv encrypt` | Encrypt `.env` → `.env.enc` (safe to commit) |
| `sealenv decrypt` | Decrypt `.env.enc` → `.env` |
| `sealenv add KEY=VALUE` | Add a secret to the active profile |
| `sealenv profile create <name>` | Create a new profile |
| `sealenv profile use <name>` | Switch active profile |
| `sealenv profile list` | List all profiles with active indicator |
| `sealenv run -- <cmd>` | Run a command with env vars injected, no file written |
| `sealenv diff <a> <b>` | Show differences between two profiles |
| `sealenv key export` | Export your key (share with teammates) |
| `sealenv key import <key>` | Import a key from a teammate |

---

## How it works

- **AES-256-GCM** encryption — authenticated, tamper-proof
- Keys stored in `~/.sealenv/keys/` — never in your repo
- `sealenv init` auto-adds `.env` and key paths to `.gitignore`
- Profiles stored in `.sealenv/profiles/` — all plaintext locally, encrypted separately
- `sealenv run` injects vars directly into the subprocess environment — nothing touches disk

---

## Team workflow

```sh
# Person A: set up and share key
sealenv init
sealenv encrypt
sealenv key export   # → prints a base64 string

# Person B: import key and decrypt
sealenv key import <base64-string>
sealenv decrypt
```

---

## Profiles example

```sh
sealenv profile create dev
sealenv profile create staging
sealenv profile create prod

sealenv profile use staging
sealenv diff dev staging   # see what changed
sealenv run -- node deploy.js
```

---

## Contributing

Issues and PRs welcome. Please open an issue before working on large changes.

```sh
git clone https://github.com/sozenka/sealenv
cd sealenv
cargo build
cargo test
```

---

## Sponsor

If `sealenv` saves you time or protects your secrets, consider sponsoring:

**[github.com/sponsors/sozenka](https://github.com/sponsors/sozenka)**

Company using sealenv in production? [Reach out](https://github.com/sozenka/sealenv/issues) about priority support.
