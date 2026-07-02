# Speedrun

A desktop and mobile study app built on [Anki](https://apps.ankiweb.net), focused on one exam (the MCAT). Anki tells you what you will remember. Speedrun also separates what you can apply and what you would score, and it teaches application directly instead of leaving it to passage volume.

Speedrun is a fork of Anki and keeps Anki's Rust scheduling engine, so the same engine runs on the desktop app and on the phone.

## What it adds

- **Authoring.** A deck is organized as a topic hierarchy of concepts, each with multiple-choice problems. A Decks home, a hierarchy editor, and a concept editor replace the classic deck browser.
- **A staged study model.** Each concept moves through Learning, Practicing, Applying, and Mastering. Learning teaches a topic as one block (two contrasting problems, then the concept); later stages recall the concept and solve its problems, with a principle-first scaffold that fades. Scheduling is real FSRS.
- **A bespoke study screen.** A full-window flow with a New Topic intro that walks down the topic tree, a per-card mastery indicator, an inline difficulty rating where a Next button would sit, and an animation when a card is upgraded.
- **Three separate scores.** Memory, Performance, and Readiness, each shown with its range, topic coverage, and the rule that makes it abstain when the data is thin. They are never blended into one number.
- **Demo mode.** Tools menu, then "Speedrun: Demo study screens", walks every study screen with mock data and no cards.

## Running on desktop

Every task is a `just` recipe (see the `justfile` and `CLAUDE.md`):

- `just run` builds and launches the desktop app.
- `just check` runs the build, tests, and lints.
- `just web-watch` live-reloads the web frontend while developing.

## Running on the phone (AnkiDroid)

The phone app is a fork of AnkiDroid that runs this repository's Rust engine, so the whole Speedrun UI (decks, overview, authoring, dashboard, study) renders on the phone from the same code. It lives in two sibling repos cloned next to this one: `Anki-Android` (the app) and `Anki-Android-Backend` (the engine bridge, whose `anki` submodule points back at this checkout). Design notes are in [`docs/plan/spec-mobile-shared-engine.md`](./docs/plan/spec-mobile-shared-engine.md).

Prerequisites: the Android SDK with an emulator or a connected device, the NDK version named in `Anki-Android-Backend/gradle/libs.versions.toml`, and JDK 17 or newer. Point `Anki-Android-Backend/anki` at this checkout, and set `local_backend=true` in `Anki-Android/local.properties`.

1. Build the engine bridge (`.aar`); this cross-compiles this repo's Rust for Android:

```bash
cd Anki-Android-Backend
export ANDROID_HOME="$HOME/Library/Android/sdk"          # your SDK location
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/<version>"    # version from libs.versions.toml
export JAVA_HOME=<path-to-jdk-17+>
./build.sh
```

2. Build the app APK:

```bash
cd ../Anki-Android
JAVA_HOME=<path-to-jdk-17+> ./gradlew assembleFullDebug
```

3. Install and launch on a running emulator or device (arm64 shown; use the ABI that matches your target):

```bash
adb install -r -g AnkiDroid/build/outputs/apk/full/debug/AnkiDroid-full-arm64-v8a-debug.apk
adb shell appops set com.ichi2.anki.debug MANAGE_EXTERNAL_STORAGE allow   # first run only
adb shell am start -n com.ichi2.anki.debug/com.ichi2.anki.IntentHandler
```

The app opens to the Speedrun Decks home. A study session needs a deck with due cards (authored on desktop and synced to the device), so a fresh install shows "Nothing due" until content is present.

## Design docs

Product and engineering decisions live in [`docs/plan/`](./docs/plan): the [PRD](./docs/plan/prd-speedrun.md), the [decision log](./docs/plan/decisions.md), the [requirements ledger](./docs/plan/requirements.md), and the specs for the study model, mastery progression, topic taxonomy, scores, and the engine queue.

## Upstream and license

Speedrun is a fork of Anki. For the upstream project, its contributors, and build/development documentation:

- Anki: https://apps.ankiweb.net
- Contributors: [CONTRIBUTORS](./CONTRIBUTORS)
- Development: [docs/development.md](./docs/development.md)

License, unchanged from Anki (AGPLv3): [LICENSE](./LICENSE)
