# Cue

A language for writing world interactions, dialogue, and quests on Roblox.

Designers write Cue. A Rust compiler generates Luau, and a small runtime connects
the result to the game's event bus. Programmers expose game capabilities;
designers decide how they fit together.

Cue is in early development. Scenes support typed locals, records, enums,
lists, conditions, and bounded loops. Game capability contracts are next.

```text
actor MECHANIC "mechanic"

scene INTRO {
    [MECHANIC] "The lighthouse needs a few repairs."
    [MECHANIC] "Come back when you've found the parts."
}
```

## Building

Install [Rust](https://rustup.rs), then run:

```sh
cargo build --locked --workspace
cargo run -p cue-cli -- build examples/hello.cue -o build/hello.luau
```

Install the CLI locally with `cargo install --locked --path crates/cue-cli`.
Development builds are available from [GitHub Releases](https://github.com/siriuslatte/cue/releases/tag/dev).

## Runtime

The runtime lives in [`runtime/`](runtime/). Connect its events to your dialogue
UI and acknowledge each line when the player advances. See
[`examples/hello.luau`](examples/hello.luau) for the integration.

## Documentation

Start with [`docs/`](docs/README.md) for the compiler, runtime, and current syntax.
Documentation will move to GitHub Pages as the language develops.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for local setup and checks.

## License

[MIT](LICENSE).
