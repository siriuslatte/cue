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
| `tests/` | Runtime and compiler-to-runtime checks using Lune |
| `docs/` | Language and architecture notes |
| `scripts/` | Release packaging |

Keep compiler logic in the library. Add crates when a component needs a separate
dependency boundary, rather than creating a crate for every compiler pass.

## Checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
stylua --check runtime/src examples tests
luau-analyze runtime/src/init.luau
lune run tests/runtime.luau
cargo run --locked -p cue-cli -- build examples/hello.cue -o build/hello.luau
lune run tests/integration.luau build/hello.luau
rojo build runtime/default.project.json -o build/Cue.rbxm
```

Use `lune run examples/hello.luau` to run the example after compiling it.
Lune tests exercise real Luau execution; Roblox service integration still needs
Studio tests when adapters start using engine APIs.

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
