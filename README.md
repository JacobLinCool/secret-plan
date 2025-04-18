# Secret Plan – Cross‑Platform Password Manager

Secret Plan is a zero‑knowledge password manager delivered as a single code‑base desktop + mobile application via **Tauri 2**. All secrets are protected locally with **Argon2 key‑derivation → AES‑256‑GCM vault encryption**; only ciphertext is ever synchronised. Key goals include leak monitoring with the HIBP API, auto‑fill on iOS/Android/desktop browsers, multi‑factor unlock, and fine‑grained audit logs.

---

## Table of Contents

- [Secret Plan – Cross‑Platform Password Manager](#secret-plan--crossplatform-password-manager)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Architecture](#architecture)
  - [Data Storage](#data-storage)
  - [UI Overview](#ui-overview)
  - [Development Setup](#development-setup)
    - [Prerequisites](#prerequisites)
    - [Install dependencies](#install-dependencies)
    - [Init mobile development environment](#init-mobile-development-environment)
    - [Run in development mode](#run-in-development-mode)
    - [Build for production](#build-for-production)
  - [Project Structure](#project-structure)
  - [Contribution Guide](#contribution-guide)
    - [Branching](#branching)
    - [Coding Standards](#coding-standards)
    - [Testing](#testing)
    - [Issues \& Roadmap](#issues--roadmap)
  - [Known Gaps \& TODOs](#known-gaps--todos)
  - [References](#references)

---

## Features

- **Zero-knowledge encryption:** Master password never leaves device; all vault data is encrypted with Argon2 + AES-256-GCM.
- **Cross-platform:** Desktop (macOS, Windows, Linux) and mobile (iOS, Android) via Tauri 2.
- **Modern UI:** Built with Svelte 5 and Tailwind 4.
- **Credential CRUD:** Add, edit, delete, and search credentials.
- **Password generator:** (Partial) Secure password generation.
- **Leak monitoring:** HIBP API integration for breach checks.
- **Audit log:** Immutable log for security review.
- **Planned:** Auto-fill, multi-factor unlock, sync & backup, security analytics, undo/redo.

---

## Architecture

```
Frontend (Svelte 5 + Tailwind 4)
    ↓ IPC
Core (Tauri Rust)
    - Vault Manager (AES-GCM, Argon2)
    - Encrypted SQLite DB
    - HIBP Client
    - (Planned) Sync Service, OS Integration
Cloud (optional, planned)
    - Object Store (R2/S3/WebDAV)
```

- **Frontend:** Svelte 5 runes for reactivity, Tailwind 4 for styling.
- **Core:** Rust (Tauri) handles all cryptography and database logic. UI never touches plaintext.
- **Sync:** Planned for future release.

---

## Data Storage

| Table         | Columns                                                                                                                                                                                    | Notes                                                        |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------ |
| `meta`        | `key TEXT PK`, `value BLOB NOT NULL`, `nonce BLOB`                                                                                                                                         | Encrypted app settings (argon params, theme, etc.)           |
| `vault_items` | `uuid TEXT PK`, `site TEXT`, `username TEXT`, `secret_enc TEXT`, `tags TEXT`, `created_at INTEGER`, `updated_at INTEGER`, `expires_at INTEGER`, `strength INTEGER`, `breach_state INTEGER` | `secret_enc` is base64-encoded JSON container (nonce+cipher) |
| `audit_log`   | `id INTEGER PK AUTOINCREMENT`, `timestamp INTEGER`, `action TEXT`, `item_uuid TEXT`                                                                                                        | Immutable log for security review                            |

- All tables live in **SQLite** (sqlcipher/rust-sqlite) with page-level AES-GCM.
- Row-level random IVs prevent pattern leakage.

---

## UI Overview

| Screen              | Primary Regions                                                               | Interaction Hints (Tailwind 4)                        |
| ------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------- |
| **Unlock**          | Centered card – password field, Touch ID button.                              | Shake‑on‑error via `animate‑shake`.                   |
| **Dashboard**       | Side bar (tags/folders); main grid of credential cards; top bar search.       | Hover‑scale on cards, strength‑meter colored bar.     |
| **Edit Item Modal** | Dialog overlay; form groups with floating labels; password generator drawer.  | Drawer slides in from right.                          |
| **Settings**        | Tabs: “Security”, “Sync”, “Appearance”, “About”. Toggle switches, prose text. | Toggle styling via `data-[state=checked]:bg-primary`. |

---

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (nightly recommended)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (recommended)
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites/)
- [SvelteKit](https://kit.svelte.dev/) (handled via dependencies)

### Install dependencies

```sh
pnpm install
```

### Init mobile development environment

Requires Xcode or Android Studio.

```sh
pnpm tauri ios init
# or
pnpm tauri android init
```

### Run in development mode

```sh
pnpm tauri dev
# or
pnpm tauri ios dev
# or
pnpm tauri android dev
```

- This will start both the SvelteKit frontend and Tauri backend.

### Build for production

```sh
pnpm tauri build
```

---

## Project Structure

```
/src-tauri/                 # Tauri Rust backend
  ├── src/
  │   ├── crypto.rs         # Cryptography (Argon2, AES-GCM)
  │   ├── error.rs          # Error types
  │   ├── hibp.rs           # HIBP API client
  │   ├── lib.rs            # Rust lib entry
  │   ├── main.rs           # Tauri main
  │   ├── models.rs         # Data models
  │   ├── sqlite_repo.rs    # SQLite repository
  │   ├── strength.rs       # Password strength
  │   ├── traits.rs         # Traits for repositories/services
  │   └── vault.rs          # Vault manager
  ├── Cargo.toml            # Rust dependencies
  └── ...
/src/                     # SvelteKit frontend
  └── ...
/package.json             # Frontend dependencies & scripts
```

---

## Contribution Guide

### Branching

- Use feature branches: `feature/<short-desc>`, `fix/<short-desc>`, etc.
- PRs should be atomic and focused.

### Coding Standards

- **Rust:** Follow [rustfmt](https://github.com/rust-lang/rustfmt) and [clippy](https://github.com/rust-lang/rust-clippy) recommendations.
- **Svelte/JS:** Use Prettier and ESLint (if configured).
- **Security:** Never log or expose plaintext secrets in UI or logs.

### Testing

- Rust: `cargo test` (see `/src-tauri/tests.rs`)
- JS: (Planned) Add SvelteKit component and integration tests.

### Issues & Roadmap

- See [Known Gaps & TODOs](#known-gaps--todos) below.
- File issues for bugs, feature requests, or questions.

---

## Known Gaps & TODOs

- **SyncService, envelope encryption, and cloud backup:** Not yet implemented.
- **Biometric/multi-factor unlock:** Not yet implemented.
- **Password generator builder pattern:** Not yet implemented.
- **Undo/redo (Command pattern):** Not yet implemented.
- **Audit log coverage:** Not all actions are consistently logged.
- **Autofill/OS integration:** Not yet implemented.
- **Security analytics (expiry, reuse, reminders):** Not yet implemented.

---

## References

- [Tauri](https://tauri.app/)
- [Svelte](https://svelte.dev/)
- [Tailwind CSS](https://tailwindcss.com/)
- [Argon2](https://github.com/P-H-C/phc-winner-argon2)
- [AES-GCM](https://en.wikipedia.org/wiki/Galois/Counter_Mode)
- [Have I Been Pwned API](https://haveibeenpwned.com/API/v3)
