# Intro

This project aims to build a **Tauri v2 plugin** that bridges a Tauri-based app (desktop or mobile) with the **AnkiDroid API** on Android.  
The ultimate goal is to allow a Tauri app (including webview/JS code) to talk directly to AnkiDroid in a safe and permissioned way—adding notes, syncing, and interacting with decks—without requiring external HTTP servers or manual intent handling.  

At the bootstrap stage, the focus is **infrastructure**:
- A cross-language repo (Rust plugin + Kotlin Android module + JS bindings).
- Reproducible **Linux-only dev environment** with GUI Android Emulator.
- Scripts to **auto-provision an emulator with AnkiDroid pre-installed**, so every dev works with the same baseline.
- A “Hello World” round-trip command through Rust ↔ Kotlin ↔ JS ↔ Demo app.

---

## Key Technologies

- **[Tauri v2](https://v2.tauri.app/)**  
  Cross-platform application framework using Rust as the backend and a webview for the frontend. v2 adds first-class support for **mobile plugins** (Android/iOS).

- **[Tauri Plugins](https://v2.tauri.app/develop/plugins/)**  
  Reusable modules that extend Tauri apps. Plugins can include native code (Rust, Kotlin, Swift) and expose commands callable from JS (`invoke("plugin:name|command")`).

- **[AnkiDroid](https://github.com/ankidroid/Anki-Android)**  
  Open-source flashcard app for Android. It exposes APIs (Instant-Add API, ContentProvider, intents) for programmatic note creation and syncing.

- **[AnkiConnect](https://foosoft.net/projects/anki-connect/)** (desktop-only fallback, future roadmap)  
  JSON API add-on for Anki desktop that listens on localhost.

- **[Android Emulator](https://developer.android.com/studio/run/emulator)**  
  Runs Android images for dev/testing. Supports hardware acceleration with KVM (Linux). In this project, the emulator is booted inside a devcontainer with GUI forwarding.

- **[Dev Containers](https://containers.dev/)**  
  A portable development environment spec (used by VS Code, GitHub Codespaces). We use it to unify Rust, Node, Java, Android SDK, and emulator dependencies.

- **[Rust](https://www.rust-lang.org/)**  
  Safe systems programming language; primary implementation of Tauri plugins.

- **[Kotlin](https://kotlinlang.org/)**  
  Official language for Android development. Used here for the Android side of the Tauri plugin.

- **[TypeScript](https://www.typescriptlang.org/)**  
  Typed superset of JS used for the bindings package. Provides a clean API for web apps to talk to the plugin.

- **[pnpm](https://pnpm.io/)**  
  Fast JS package manager. Used to manage JS bindings and demo app.

---

## Research / Reference Links

- Tauri v2  
  - https://v2.tauri.app/  
  - https://v2.tauri.app/develop/plugins/  
  - https://v2.tauri.app/reference/javascript/

- Tauri Mobile (Android/iOS)  
  - https://v2.tauri.app/start/prerequisites/#mobile  
  - https://v2.tauri.app/develop/plugins/mobile/

- AnkiDroid APIs  
  - https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API  
  - https://github.com/ankidroid/Anki-Android/wiki/AnkiDroid-API-(v1.1.0-2021-09-15)

- AnkiConnect (desktop)  
  - https://foosoft.net/projects/anki-connect/  
  - https://ankiweb.net/shared/info/2055492159

- Android Emulator docs  
  - https://developer.android.com/studio/run/emulator  
  - https://developer.android.com/studio/run/emulator-acceleration

- Dev Containers  
  - https://containers.dev/  
  - https://code.visualstudio.com/docs/devcontainers/containers



Objective
---------
Stand up a Tauri v2 plugin repo with:
- Cross-language skeleton (Rust plugin + Android Kotlin module + JS/TS bindings + demo app)
- Linux-only dev flow with GUI Android Emulator (via your devcontainer)
- Reproducible emulator provisioning (AnkiDroid auto-install)
- "Hello World" verification path (round-trip invoke)
- Minimal but meaningful tests (unit + integration smoke)
- No real Anki API yet—just wiring, permissions, and stubs


Repo Structure (no code here, just shape)
-----------------------------------------
tauri-plugin-ankidroid/
  packages/
    tauri-plugin-ankidroid/        # Rust plugin crate
      permissions/                 # v2 permission TOMLs
      mobile/android/              # Kotlin module (Gradle)
    plugin-ankidroid-js/           # JS/TS bindings
  examples/demo-app/               # Tauri v2 sample app
  .devcontainer/                   # (You already added these)
  scripts/
    emu-create-and-start.sh        # (You added this)
    emu-install-ankidroid.sh       # (You added this)
  justfile (optional)
  README.md


Development Workflow (Linux)
----------------------------
1) Open repo in your devcontainer (you already created config).
   - Ensure host has: /dev/kvm, X11 available (DISPLAY), and you’ve run `xhost +local:` once.

2) Start the GUI emulator inside the container:
   - `bash scripts/emu-create-and-start.sh`
   - Wait for "Emulator booted."

3) Install a pinned AnkiDroid into the emulator:
   - `bash scripts/emu-install-ankidroid.sh`
   - Confirms `org.ankidroid.android` exists.

4) Install/build workspace deps (single workspace):
   - `pnpm -w i`
   - `pnpm -w build`  (build TS bindings; Rust builds when demo-app runs)

5) Run the demo app on the emulator:
   - `pnpm --filter demo-app android:dev`
   - Tap the demo button → expect an alert/string coming from the plugin.

(Desktop smoke is optional: `pnpm --filter demo-app dev`)


Hello World Scope (what “done” means)
-------------------------------------
- JS binding exposes: `hello(name)` → returns a string
- Rust plugin registers command `"hello"` with plugin identifier `"ankidroid"`
- Android Kotlin plugin exposes matching `@Command hello(name: String)`; Rust routes to it on Android builds
- Demo app calls `hello("World")` via `invoke("plugin:ankidroid|hello", { name })` and shows the result
- Permissions: a TOML that allows `hello` in demo config
- Tests pass (see below)


High-Level Implementation Notes (pseudocode only)
-------------------------------------------------
Rust Plugin (tauri-plugin-ankidroid):
- register plugin:
  plugin("ankidroid")
    -> invoke_handler([hello])
- command:
  async hello(name: String) -> String {
    if (platform == android) {
      // either: route to mobile layer OR return stub (for bootstrap)
      return "Hello, {name} from Android!"
    } else {
      return "Hello, {name} from AnkiDroid plugin!"
    }
  }

Android Kotlin Plugin:
- @TauriPlugin class AnkiDroidPlugin : Plugin {
    @Command fun hello(name: String): String = "Hello, $name from Android!"
  }
- Gradle module is referenced by mobile build; no Anki APIs yet

JS Bindings (plugin-ankidroid-js):
- export function hello(name: string): Promise<string> {
    return invoke("plugin:ankidroid|hello", { name })
  }

Demo App:
- UI with a button that awaits hello("World") and displays the string
- Tauri config includes plugin and permission for `hello`


Permissions & Config
--------------------
- Create a minimal permission set that allows just the `hello` command
- In demo app’s `tauri.conf.json`, enable plugin `"ankidroid"` and include the permission ID
- Keep all future commands (e.g., addNotes) blocked by default until explicitly allowed


Testing Strategy (bootstrap level)
----------------------------------
1) Rust unit test (fast):
   - Test that `hello("Tester")` returns expected shape/string
   - No Android involved

2) JS unit test (fast):
   - Test that `hello` is exported and callable (mock invoke)
   - Do not require a running Tauri runtime for unit tests

3) Android unit test (Robolectric) or plain JVM test:
   - Test `AnkiDroidPlugin().hello("Tester")` contains "Tester"
   - No emulator required

4) Manual integration smoke (emulator GUI):
   - Use your scripts to boot emulator and install AnkiDroid
   - Build demo app for Android; tap button; verify round-trip value
   - (Optional) Add an adb log assertion or screenshot step later

5) (Later) Instrumented Android test (connected/emulator):
   - Nice-to-have once you add real Anki interactions


CI (short-term recommendation)
------------------------------
- Use GitHub Actions Ubuntu runners for: `cargo check/test`, `pnpm build/test`
- Skip emulator-in-CI initially (KVM is tricky on hosted). If needed, use a self-hosted Linux runner with /dev/kvm + Xvfb
- Add a job to lint Gradle + run JVM tests (Robolectric) without emulator
- Gate merges on unit tests passing and demo app building

Example CI jobs (just bullets, no YAML):
- Job A: Rust (fmt, clippy, test)
- Job B: Node (pnpm build, vitest)
- Job C: Android JVM tests (./gradlew test) for the Kotlin module


Reproducible Emulator & Anki Setup
----------------------------------
- Your `emu-create-and-start.sh`:
  - Ensures a named AVD exists (e.g., API 34 Google APIs x86_64)
  - Boots with GUI + GPU acceleration
  - Waits for sys.boot_completed

- Your `emu-install-ankidroid.sh`:
  - Downloads a pinned AnkiDroid APK (version variable)
  - `adb install -r` into the running emulator
  - Verifies package installed

- Team usage:
  - Everyone runs the same two scripts → consistent emulator baseline
  - Bump ANKI_VERSION in script to roll the environment forward
  - Consider caching the APK under `third_party/apk/` for reliability


Milestones (for the coding agent)
---------------------------------
M1 — Workspace & Wiring
- Create workspace scaffolding (crates, packages, example app)
- Implement bare plugin registration, hello command, permission TOML
- Wire JS bindings + demo app button
- Verify desktop “hello” (fast loop)

M2 — Android Path
- Ensure Android module compiles and exports `hello`
- Confirm Rust → Android routing (build demo-app for Android)
- Boot emulator (GUI) via script and run demo app; confirm alert text

M3 — Reproducible Dev Env
- Finish emulator + AnkiDroid provisioning scripts (already present)
- Document one-liners in README (emu start, install, run demo)

M4 — Tests & CI
- Add Rust + JS + Kotlin unit tests
- Add GitHub Actions jobs for non-emulator tests
- Badge + CONTRIBUTING notes

M5 — Next Steps (not for bootstrap)
- Add real AnkiDroid API integration (Instant-Add, etc.)
- Extend permission TOMLs per command
- Optional desktop AnkiConnect shim
- Versioning & publish strategy (crate + npm)


Success Criteria (exit for bootstrap)
-------------------------------------
- `pnpm --filter demo-app android:dev` deploys to the GUI emulator
- Tapping the demo button shows a string coming from the plugin (Android path)
- `cargo test`, JS tests, and Kotlin JVM tests pass in container
- `bash scripts/emu-install-ankidroid.sh` installs the pinned AnkiDroid successfully
- README explains the one-liners for the team


README Additions (for your agent to write)
------------------------------------------
- Short “Getting Started (Linux + devcontainer)” section:
  - Prereqs on host (KVM, X11, xhost)
  - Start emulator (script), install AnkiDroid (script), run demo app
- Troubleshooting:
  - “No GUI window?” → check DISPLAY bind + xhost
  - “Slow emulator?” → verify KVM, try `-gpu host` fallback flags
  - “APK download failing?” → point to cached `third_party/apk/` or mirror
