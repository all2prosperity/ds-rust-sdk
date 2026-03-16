# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo check                       # check async-only build
cargo check --features blocking   # check with blocking wrapper included
cargo build
cargo build --features blocking
cargo test
cargo test --features blocking
```

## Architecture

This is the Rust client SDK (`datasneaker-sdk`) for the DataSneaker event tracking server.

**Module layout:**

| File | Purpose |
|------|---------|
| `src/client.rs` | Async `Client` — the primary public type |
| `src/blocking.rs` | Sync wrapper `blocking::Client`; only compiled with `--features blocking` |
| `src/types.rs` | `ClientConfig`, `TrackEvent`, `EventPayload`, server response types |
| `src/error.rs` | `SdkError` enum |

**Client internals:**

- On `Client::new`, a `tokio::spawn`ed background task is started. It loops on a `tokio::select!` over a periodic timer (`flush_interval_ms`), a `Notify` triggered when the in-memory queue hits `max_batch_size`, and a shutdown `Notify`.
- `track()` pushes an `EventPayload` onto a `Mutex<VecDeque>`. If the queue reaches `max_queue_size` it returns `SdkError::QueueFull` (drop, no blocking).
- On flush failure (non-2xx or network error), events are pushed back to the front of the queue so they are retried on the next flush cycle.
- `shutdown()` consumes `self`, signals the background task, and awaits it — guaranteeing a final flush before returning.
- `blocking::Client` owns a `tokio::runtime::Runtime` and `block_on`s every async call. It uses `Option<crate::client::Client>` so `shutdown` can `take()` the inner client.

**Auth:** `X-App-Key` header is set once in `reqwest` default headers at construction time from `ClientConfig::app_key`.

**Feature flag:** `blocking` is a purely additive feature — no default features change. The async `client` module is always compiled.
