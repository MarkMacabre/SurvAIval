# SurvAIval

**Local-first, offline survival knowledge system for Android and Linux.**

## Overview

SurvAIval is a deterministic, offline emergency survival knowledge retrieval system designed to work without any internet connection. It provides step-by-step survival guidance when you need it most.

Unlike AI chatbots that can hallucinate dangerous medical advice, SurvAIval uses **dictionary-based keyword matching** and only returns verified information from trusted sources. If it doesn't have verified data, it tells you explicitly: _"I do not have verified information for this scenario."_

## Quick Start

```bash
# Build the project
cargo build --release

# Initialize database
./target/release/survaival init --db survival.db

# Load sample data (5 verified survival units)
./target/release/survaival ingest --db survival.db --file test_data/sample_units.json

# Query for guidance
./target/release/survaival query --db survival.db "bleeding now" --urgency "Immediate"
```

## Key Features

- ✅ **Fully Offline** — No network calls, no cloud, no telemetry
- ✅ **No Hallucination** — Dictionary-based matching, never fabricates advice
- ✅ **Deterministic** — Same input always produces same output
- ✅ **Single Best Match** — Returns exactly one reliable result, not a confusing list
- ✅ **Verified Sources** — All guidance from trusted medical/survival manuals
- ✅ **Instant Response** — Query results in < 5ms
- ✅ **Cross-Platform** — Linux CLI (ready) + Android app (planned)

## Documentation

- **[USAGE.md](USAGE.md)** - End-user guide with examples
- **[BUILD.md](BUILD.md)** - Build instructions and cross-compilation
- **[IMPLEMENTATION.md](IMPLEMENTATION.md)** - Architecture and Android specs

## Hard Constraints

1. ✅ No internet connectivity required or used
2. ✅ No fabrication of medical/survival guidance
3. ✅ Explicit failure messages when data is unavailable
4. ✅ Human review required before content is committed
5. ✅ Enum-only tags (no free text)
6. ✅ Minimal resource usage (runs on 1GB RAM devices)
7. ✅ Deterministic scoring algorithm
8. ✅ Single best match per query
9. ✅ No background services
10. ✅ No tracking or analytics

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    FRONTENDS                        │
│  ┌──────────────┐         ┌──────────────────┐     │
│  │   Android    │         │   Linux CLI      │     │
│  │  (planned)   │         │    (ready)       │     │
│  └──────┬───────┘         └────────┬─────────┘     │
│         │ JNI/FFI                  │ Native        │
├─────────┴──────────────────────────┴───────────────┤
│              SHARED RUST CORE ENGINE               │
│  ┌────────┐ ┌───────┐ ┌──────────┐ ┌───────────┐  │
│  │ Ingest │ │ Dict  │ │ Search & │ │    FFI    │  │
│  │        │ │       │ │  Score   │ │           │  │
│  └────────┘ └───────┘ └──────────┘ └───────────┘  │
│                       │                            │
│            ┌──────────┴──────────┐                 │
│            │  SQLite (FTS5)      │                 │
│            └─────────────────────┘                 │
└────────────────────────────────────────────────────┘
```

## Sample Data

Includes 5 verified survival units:
- **Severe bleeding control** - Tourniquet protocol from Field Medical Guide
- **CPR for cardiac arrest** - AHA Guidelines 2020
- **Choking** - Heimlich maneuver from Red Cross
- **Fracture immobilization** - Wilderness Medicine Manual
- **Burn treatment** - Emergency Care Manual

## Example Output

```
════════════════════════════════════════════════════════════════
  Control severe external bleeding
  Confidence: Verified | Score: 170
════════════════════════════════════════════════════════════════

▶ IMMEDIATE ACTION:
  1. Apply direct pressure to the wound with a clean cloth
  2. If bleeding soaks through, add more dressings — do not remove original
  3. If bleeding continues, apply a tourniquet proximal to wound and note time

⚠ WHAT NOT TO DO:
  • Do NOT remove embedded objects from the wound
  • Do NOT release tourniquet once applied

▶ SOURCE:
  Field Medical Guide v2.1, Chapter 4, Pages 12-15
```

## Safety Warning

⚠️ **IMPORTANT**: SurvAIval is NOT a replacement for professional medical care. Always seek professional help when available. This tool is for emergency situations when professional help is not immediately accessible.

## License

MIT License - See [LICENSE](LICENSE) for details.

**Disclaimer**: This software is provided "as is" without warranty. The developers are not responsible for any harm resulting from the use or misuse of this software.

## Status

✅ **Linux CLI**: Production ready
🚧 **Android App**: Architecture designed, implementation needed (see [IMPLEMENTATION.md](IMPLEMENTATION.md))