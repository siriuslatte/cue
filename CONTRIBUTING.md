# Contributing

## Setup

Install Rust and [Rokit](https://github.com/rojo-rbx/rokit), then run:

```sh
rokit install
cargo build --locked --workspace
```

Rust is pinned in `rust-toolchain.toml`. Luau tools are pinned in `rokit.toml`.

## Layout

| Path | Purpose |
| --- | --- |
| `crates/cue-compiler/` | Parsing, diagnostics, semantic checks, and Luau generation |
| `crates/cue-cli/` | File handling and command-line interface |
| `runtime/` | Luau execution and the game bus boundary |
| `examples/` | Small runnable content examples |
| `tests/` | Lest runtime and compiler-to-runtime suites |
| `docs/` | Language and architecture notes |
| `scripts/` | Release packaging |

Keep compiler logic in the library. Add crates when a component needs a separate
dependency boundary, rather than creating a crate for every compiler pass.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --workspace
stylua --check runtime/src examples tests
luau-analyze runtime/src/init.luau
lest --forbid-only
rojo build runtime/default.project.json -o build/Cue.rbxm
```

To run the example:

```sh
cargo run --locked -p cue-cli -- build examples/hello.cue -o build/hello.luau
lune run examples/hello.luau
```

## Rust tests

Rust tests use rstest for fixtures and named cases, assert_cmd and assert_fs for
CLI behavior, and Insta for compiler output and diagnostic snapshots.

Source fixtures live in `crates/cue-compiler/tests/fixtures/`. Their snapshots
live in `crates/cue-compiler/tests/snapshots/`. Review changes with `cargo insta review`
after installing the optional cargo-insta tool. Do not accept snapshot updates
without checking their contents. CI compares snapshots without updating them.

## Luau tests

[Lest](https://github.com/lest-luau/lest) is pinned in `rokit.toml`. Its framework
is generated under `.lest/` on the first run and is not committed.

- `lest run runtime` tests the runtime in Lest's native Luau VM.
- `lest run integration` invokes the compiled CLI and executes its output in Lune.
- `lest run runtime --coverage` reports runtime line coverage.

Build the CLI before running the integration suite. It uses `target/debug/cue`
(`cue.exe` on Windows); set `CUE_BIN` when using a different target directory.
CI runs both suites, rejects focused tests, and requires 90% runtime line coverage.

These suites cover runtime logic and generated Luau. Roblox service integration
will need engine tests when adapters start using engine APIs.

## Changes

Branch from `main`. Use `feature/`, `fix/`, `refactor/`, `docs/`, `test/`, `ci/`,
or `chore/` prefixes. Commits and PR titles follow Conventional Commits, for
example `feat: add dialogue choices` or `fix(runtime): reject stale steps`.

Keep PR descriptions to the change and the checks run. Update examples and docs
alongside language changes.

## Releases

CI checks pull requests and `main`. Successful main builds update the `dev`
prerelease with compiler binaries, the runtime module, source, and checksums.

For a versioned release, update the workspace and runtime package versions,
merge to `main`, and push a matching `v<version>` tag. The release workflow reruns
CI before publishing. Prerelease versions remain marked as prereleases.

Documentation stays under `docs/` until the Pages site is introduced.
