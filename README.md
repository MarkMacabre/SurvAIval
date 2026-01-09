# SurvAIval

**Local-first, offline survival knowledge system for Android and Linux.**

## Overview

SurvAIval is a deterministic, offline emergency survival knowledge retrieval system designed to work without any internet connection. It provides step-by-step survival guidance when you need it most.

## Key Features

- **Fully Offline** — No network calls, no cloud, no telemetry
- **Deterministic** — Same input always produces same output
- **Single Best Match** — Returns exactly one reliable result
- **Panic Mode** — Quick access to critical survival information
- **Voice Interface** — Text-to-speech for hands-free operation
- **Cross-Platform** — Android and Linux support

## Hard Constraints

1. No internet connectivity required or used
2. No fabrication of medical/survival guidance
3. Explicit failure messages when data is unavailable
4. Human review required before content is committed
5. Enum-only tags (no free text)
6. Minimal resource usage (runs on 1GB RAM devices)

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    FRONTENDS                        │
│  ┌──────────────┐         ┌──────────────────┐     │
│  │   Android    │         │   Linux CLI/TUI  │     │
│  └──────┬───────┘         └────────┬─────────┘     │
│         │ JNI                      │ FFI           │
├─────────┴──────────────────────────┴───────────────┤
│              SHARED RUST CORE ENGINE               │
│  ┌────────┐ ┌───────┐ ┌──────────┐ ┌───────────┐  │
│  │ Ingest │ │ Index │ │ Retrieve │ │   Voice   │  │
│  └────────┘ └───────┘ └──────────┘ └───────────┘  │
│                       │                            │
│            ┌──────────┴──────────┐                 │
│            │  SQLite (FTS5)      │                 │
│            └─────────────────────┘                 │
└────────────────────────────────────────────────────┘
```

## License

MIT License

## Status

🚧 Under Development