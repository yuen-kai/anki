# Speedrun

A desktop and mobile study app for the MCAT, built on Anki's engine. Speedrun keeps
Anki's Rust scheduling core and runs it on both the desktop app and the phone, so one
change reaches both platforms. In place of Anki's plain note and deck flow it uses an
authored topic hierarchy, a staged study model (Learning, Practicing, Applying,
Mastering), and three separate scores (Memory, Performance, Readiness) that abstain
when there is not enough data to be honest.

Speedrun is a fork of Anki; the Android app is a fork of AnkiDroid. Both are vendored
into this one repository and are no longer tracked against the upstream projects.

## Repository layout

One repo, four parts. The engine and the web UI are shared; each platform is a thin
host that embeds them. See [`ProjectDocs/CODEBASE.md`](./ProjectDocs/CODEBASE.md) for
the full code map.

- `rslib/`: the shared Rust engine (scheduling/FSRS, storage, the protobuf API).
- `ts/`: the shared SvelteKit UI (the `speedrun-*` routes).
- `qt/` and `pylib/`: the desktop host (the PyQt app and its Python bridge to the engine).
- `android/app` and `android/bridge`: the Android host (the AnkiDroid app and rsdroid,
  the JNI bridge that compiles the engine and packages it into an `.aar`).

## Running the desktop app

Everything is a `just` recipe (run `just --list` for the full set):

- `just run` builds pylib and qt and launches the app.
- `just check` runs the full build, tests, and lints.
- `just web-watch` live-reloads the web UI during development.

See [docs/development.md](./docs/development.md) for the base build setup.

## Building the Android app

Prerequisites: the Android SDK and NDK, a JDK 17 or newer, and an arm64 emulator or
device. Point `JAVA_HOME` at the JDK and `ANDROID_NDK_HOME` at the NDK, and create two
machine-local files:

- `android/app/local.properties` with `sdk.dir=<your SDK path>` and `local_backend=true`
- `android/bridge/local.properties` with `sdk.dir=<your SDK path>`

Build the engine bridge first. This cross-compiles the Rust engine and bundles the
shared web UI into an `.aar`:

```bash
cd android/bridge
cargo run -p build_rust
```

The dev build is a single architecture (arm64 on Apple Silicon). For a multi-arch
release `.aar`, run `ALL_ARCHS=1 RELEASE=1 cargo run -p build_rust`.

Then build and install the app on a running arm64 emulator or device:

```bash
cd ../app
./gradlew assembleFullDebug
adb install -r -g AnkiDroid/build/outputs/apk/full/debug/AnkiDroid-full-arm64-v8a-debug.apk
adb shell am start -n com.ichi2.anki.debug/com.ichi2.anki.IntentHandler
```

On the first install only, grant storage access:

```bash
adb shell appops set com.ichi2.anki.debug MANAGE_EXTERNAL_STORAGE allow
```

The app opens to the Speedrun decks screen. Content is authored on the desktop app and
synced to the device, so a fresh install shows nothing due until a deck with due cards
is present.

## License

Speedrun keeps the licenses of the projects it forks (Anki and AnkiDroid), AGPLv3:
[LICENSE](./LICENSE).
