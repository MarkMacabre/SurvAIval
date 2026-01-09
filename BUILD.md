# SurvAIval Build Guide

## Quick Start

### Prerequisites
- Rust 1.70+ (install from https://rustup.rs)
- C compiler (gcc or clang)
- SQLite3 development headers (usually pre-installed)

### Build Commands

```bash
# Build in release mode (optimized)
cargo build --release

# Build in debug mode (faster compilation, includes debug symbols)
cargo build

# Run tests
cargo test

# Run the CLI
./target/release/survaival --help
```

## Build Artifacts

After running `cargo build --release`, you'll find:

### Linux CLI Binary
- **Location**: `target/release/survaival`
- **Type**: Standalone executable (3.9 MB)
- **Platform**: Linux x86_64
- **Usage**: Command-line interface for managing and querying survival data

### Shared Library (for Android JNI)
- **Location**: `target/release/libsurvaival_core.so`
- **Type**: Dynamic shared library (3.2 MB)
- **Platform**: Linux x86_64
- **Usage**: Can be loaded by Android via JNI (needs cross-compilation for ARM)

### Static Library
- **Location**: `target/release/libsurvaival_core.a`
- **Type**: Static archive (28 MB, unstripped)
- **Platform**: Linux x86_64
- **Usage**: Link directly into other programs

### Rust Library
- **Location**: `target/release/libsurvaival_core.rlib`
- **Type**: Rust-specific library format (2.0 MB)
- **Usage**: Used when linking with other Rust crates

## CLI Usage Examples

### Initialize Database
```bash
./target/release/survaival init --db survival.db
```

### Ingest Verified Survival Units
```bash
./target/release/survaival ingest --db survival.db --file test_data/sample_units.json
```

### Query for Survival Guidance
```bash
# Basic query
./target/release/survaival query --db survival.db "bleeding emergency"

# Query with available tools
./target/release/survaival query --db survival.db "bleeding now" \
    --tools "Tourniquet,Bandage" \
    --urgency "Immediate"

# Query for CPR guidance
./target/release/survaival query --db survival.db "no pulse not breathing" \
    --urgency "Immediate"
```

## Cross-Compilation for Android

To build for Android devices, you need to cross-compile for ARM and x86 architectures.

### Setup Android Targets

```bash
# Install Android targets
rustup target add aarch64-linux-android      # ARM64
rustup target add armv7-linux-androideabi    # ARM32
rustup target add i686-linux-android         # x86
rustup target add x86_64-linux-android       # x86_64
```

### Install Android NDK

1. Download Android NDK from https://developer.android.com/ndk/downloads
2. Extract to a location (e.g., `~/android-ndk-r26b`)
3. Set environment variables:

```bash
export NDK_HOME=~/android-ndk-r26b
export PATH=$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
```

### Configure Cargo for Cross-Compilation

Create `.cargo/config.toml`:

```toml
[target.aarch64-linux-android]
ar = "aarch64-linux-android-ar"
linker = "aarch64-linux-android24-clang"

[target.armv7-linux-androideabi]
ar = "arm-linux-androideabi-ar"
linker = "armv7a-linux-androideabi24-clang"

[target.i686-linux-android]
ar = "i686-linux-android-ar"
linker = "i686-linux-android24-clang"

[target.x86_64-linux-android]
ar = "x86_64-linux-android-ar"
linker = "x86_64-linux-android24-clang"
```

### Build for Android

```bash
# Build for ARM64 (most modern Android devices)
cargo build --target aarch64-linux-android --release

# Build for ARM32 (older Android devices)
cargo build --target armv7-linux-androideabi --release

# Build for x86_64 (emulators)
cargo build --target x86_64-linux-android --release

# Build for x86 (older emulators)
cargo build --target i686-linux-android --release
```

### Copy Libraries to Android Project

```bash
# Create JNI library directories
mkdir -p android/app/src/main/jniLibs/{arm64-v8a,armeabi-v7a,x86,x86_64}

# Copy built libraries
cp target/aarch64-linux-android/release/libsurvaival_core.so \
   android/app/src/main/jniLibs/arm64-v8a/

cp target/armv7-linux-androideabi/release/libsurvaival_core.so \
   android/app/src/main/jniLibs/armeabi-v7a/

cp target/i686-linux-android/release/libsurvaival_core.so \
   android/app/src/main/jniLibs/x86/

cp target/x86_64-linux-android/release/libsurvaival_core.so \
   android/app/src/main/jniLibs/x86_64/
```

## Performance

### Release vs Debug Builds

| Build Type | Size  | Speed | Use Case |
|------------|-------|-------|----------|
| Debug      | ~10MB | 1x    | Development, debugging |
| Release    | ~4MB  | 10x+  | Production deployment |

### Query Performance

- **Database initialization**: < 10ms
- **Single query**: < 5ms (after dictionary loading)
- **Dictionary loading**: ~1ms (one-time cost)
- **Memory usage**: ~5-10 MB

### Optimization Flags

The release build uses these optimizations (configured in Cargo.toml):

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single compilation unit for better optimization
strip = false       # Keep symbols for debugging if needed
```

To create a minimal binary, you can add stripping:

```bash
cargo build --release
strip target/release/survaival
strip target/release/libsurvaival_core.so
```

This reduces binary size by ~30-40%.

## Troubleshooting

### Build Errors

**Problem**: `error: linking with cc failed`
**Solution**: Install build essentials:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"
```

**Problem**: `error: could not find native static library sqlite3`
**Solution**: The bundled SQLite feature is enabled, so this shouldn't happen. If it does:
```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# Fedora/RHEL
sudo dnf install sqlite-devel
```

### Runtime Errors

**Problem**: `SIGSEGV` or segmentation fault
**Solution**: Likely an issue with FFI boundary. Ensure:
- Strings are valid UTF-8
- Pointers are not null
- Memory is properly managed across FFI boundary

**Problem**: Query returns "No match" for valid query
**Solution**: Check that:
- Database has been initialized (`init` command)
- Survival units have been ingested (`ingest` command)
- Query includes urgency keywords (now, immediately, soon, etc.)

## Distribution

### Linux Binary Distribution

The `target/release/survaival` binary can be distributed as-is. It's dynamically linked to system libraries (libc, libm, libdl, libpthread), which are available on all Linux systems.

For maximum compatibility, build on an older Linux distribution (e.g., Ubuntu 18.04) and it will run on newer systems.

### Android APK Distribution

After building the Android app with the native libraries:

```bash
cd android
./gradlew assembleRelease
```

The APK will be at: `android/app/build/outputs/apk/release/app-release.apk`

Sign it for distribution:
```bash
jarsigner -verbose -sigalg SHA256withRSA -digestalg SHA-256 \
    -keystore my-release-key.keystore \
    app-release.apk alias_name
```

## Database Management

### Database File Location

- **Linux**: Recommended in `~/.local/share/survaival/survival.db`
- **Android**: `/data/data/org.survaival.app/databases/survival.db`

### Database Backup

The database is a single SQLite file. To backup:

```bash
cp survival.db survival.db.backup
```

To compress for storage:

```bash
gzip -k survival.db  # Creates survival.db.gz
```

### Database Migration

When upgrading, the schema migrations are applied automatically on first open. To manually migrate:

```bash
sqlite3 survival.db < migrations/001_initial_schema.sql
```

## License

MIT License - See LICENSE file for details.
