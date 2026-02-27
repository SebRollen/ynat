# YNAT

You Need a TUI. Yes, you do. We all need more TUIs in our lives! TUIs
are fast, available (assuming you live in the terminal), and built for
power-users. That shiny web-app might feel exciting when you first
interact with it, but the TUI lets you get real work done.

Specifically, as you have already guessed from the title, this is a TUI
for YNAB, the budgeting app. I've been using YNAB for over a decade,
logging every single transaction during this time. In short: I'm a fan.
However, I find that accessing the web app to add transactions is a
source of friction, as the amount of data in my account has resulted in
very long loading times.

YNAT resolves this issue for me by meeting me where I most-usually am:
in the terminal, and through incremental API updates and caching lets me
get my work done faster than it would take me to load the start page of
YNAB.

## Core features

- **Budgets, accounts, transactions, and budget planning** — the full set of
  screens you need for day-to-day YNAB use
- **Fast startup via local caching** — data is cached on disk and delta-synced
  on each launch so the TUI is responsive even with years of transaction history
- **Create, edit, and delete transactions** — with autocomplete for payees and
  categories, math expression support in amount fields, split-transaction
  support, flag colours, and cleared/approved status toggles
- **Budget planning** — view and edit monthly category allocations, navigate
  between months, and filter categories by funding status (underfunded,
  overfunded, money available, etc.)
- **Reconcile accounts**
- **Real-time filtering** — filter transactions or accounts by any field with
  instant results
- **Vim-style keyboard navigation** — `hjkl`, `gg`/`G`, and multi-key sequences
  throughout; press `?` for context-sensitive help

## Non-goals / #wont-change
- Integration with plain-text accounting tools (I'm a fan, but for now I
  want to focus on the YNAB integration as that's what I use)
- Creating / deleting categories & category groups. Would love to do
  this, but the YNAB API doesn't support it
- Manually matching / unmatching transactions. Again, would love to
  support, but the YNAB API does not allow it

## Contributing and AI use

Contributions are very welcome! This project is partially developed with the
help of LLMs, and LLM-assisted pull requests are welcome too — just disclose
that you used one in your PR description. The project is MIT licensed.

## Setup

### Prerequisites

- [Rust toolchain](https://rustup.rs)

### Install

Install directly from GitHub without cloning the repository:

```bash
cargo install --git https://github.com/SebRollen/ynat
```

Or clone and build manually:

```bash
cargo build --release
./target/release/ynat
```

### First-time authentication

YNAB does not support the device authentication flow, so YNAT uses a small
intermediary OAuth server to complete the authorization. A public instance is
hosted at `https://ynat-auth-server.fly.dev` for convenience; it is completely
stateless and stores no PII — the source code for it lives in
[`ynat-auth/`](./ynat-auth) if you'd like to verify or self-host.

On the first launch YNAT will walk you through a browser-based OAuth flow to
authorise the app with your YNAB account. No configuration file is required —
YNAT connects to the hosted auth server by default. After authorising, your
token is stored in your XDG cache directory (e.g. `~/.cache/ynat/token.json`
on Linux and macOS) and refreshed automatically on subsequent launches.

If you want to self-host the auth server or point YNAT at a different instance,
create a `config.toml` next to the binary (or set `YNAB_TUI_CONFIG` to its
path):

```toml
[auth]
server_url = "https://your-auth-server.example.com"
```
