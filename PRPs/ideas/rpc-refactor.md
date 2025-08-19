# RPC Refactor

## Overview

We are refactoring our WIP Tauri plugin project to migrate away from Rust's JNI calls to a more modular and easier to maintain architecture.

## Strategy

We're having problems with the current architecture as handling all this within rust is getting too complex.

We had this working before in an old project that used Capacitor. See `PRPs/planning/REFERENCE_IMPLEMENTATION.md` for a reference implementation of the old Capacitor implementation. Doing the business logic within Java/Kotlin will make things easier to maintain, especially with error handling and Java language features, but we still need a rust layer to integrate with Tauri.

The key diagram to understand our new prposed architecture is below:

```
┌─────────────────┐
│   JavaScript    │  Frontend API
│  (TypeScript)   │  Clean, typed interface
└────────┬────────┘
         │ JSON RPC v2
┌────────▼────────┐
│      Rust       │  Thin RPC Proxy
│  (rpc_proxy.rs) │  Single entry point
└────────┬────────┘
         │ JSON RPC v2
┌────────▼────────┐
│     Kotlin      │  Business Logic
│(AnkiDroidPlugin)│  All Android/AnkiDroid interaction
└─────────────────┘
```

It's extremely important to understand that the Rust layer is very thin and does not need to know about our API. We're only using it to support Tauri. By leveraging the RPC API, we can keep the rust layer as a simple proxy. Our Ankidroid API only need to be defined in Typescript and Kotlin layers.

Eventually, we want to implement all of the default AnkiDroid RPC methods, but for now just focus on Card CRUD.

We probably want want to extend some helper methods in Kotlin layer, like ensuring a deck id is valid, or ensuring a model id is valid. Study the REFERENCE_IMPLEMENTATION.md file for an example.

See `PRPs/planning/API.md` for the proposed full JS API - we will not implement all of this yet, but it should give you an idea of where we are going, and again, not a final spec.

The JS api will include the full method names and expose typed parameters, but under the hood will use the RPC API we're creating.

## Goals

The primary goal is maintainability and readability.

We want to ensure that we keep file sizes down, and not have mega files, so we want modular files, perhaps one for each method.

Split the `-android` package into `tauri-plugin-ankidroid-rust` and `tauri-plugin-ankidroid-kotlin`.

Focus on keeping things simple.

We want to ensure coverage with unit tests and end to end tests. This will help us develop faster and more confidently.

We can save e2e tests for a later pass, which will use the `e2e-test-app` package. For now, include unit tests for the android and kotlin packages.

For the first pass, keep the scope limited to card CRUD. We will need to fetch a model id and deck id to enable this, probably.

We will build out other features later (such as media handling, deck management, etc.)

During the refactor, get rid of all superflous/dead code, across all of our packages.

We need to include a strong verification loop for each package, utilizing tests and the `npm run quickfix` scripts made available in each package. For the new packages, include similar scripts that exist currently.

Make sure we are target builds to deploy only on mobile. We don't need to deploy to desktop. Ensure dev deployments are the correct architecture.

## Plan

A high level overview of the plan:

1. Create the new `tauri-plugin-ankidroid-rust` and `tauri-plugin-ankidroid-kotlin` packages, including relevant logic.
2. Rename `tauri-plugin-ankidroid-js` to `tauri-plugin-ankidroid-typescript`.
3. Ensure unit tests are written code paths.
4. Update frontend to use the new packages.
5. Ensure we can build and run the app.
6. Write basic happy-path e2e tests using the `e2e-test-app` package. We have had problems with this with simulating button pressing etc, so we will need to figure out a reliable way to do this.

In total we should have 4 packages:

- `tauri-plugin-ankidroid-typescript`
- `tauri-plugin-ankidroid-rust`
- `tauri-plugin-ankidroid-kotlin`
- `tauri-plugin-ankidroid-e2e-test-app`

We will need to update the `package.json` files in each package to reflect the new structure.
