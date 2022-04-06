# Kurt

A language for fun.

This project is just starting.

## Install

Clone this repository with `git` and use `cargo` to build and install.

While in development, the most recent version of `stable` is targeted.

```sh
git clone https://github.com/isaacazuelos/kurt
cd kurt
# cargo test --all # if you want to run the tests first
cargo install --path=.
```

## Usage

Not a lot is supported yet. Even the `--help` isn't very informative at the
moment. There are `repl`, `eval` and `script` subcommands, which can take a
`--trace` flag to show what the VM is doing, or a `--dump` flag to show you
the compiled byte code.

## Contributing

Contributions are not yet welcome. Maybe one day!

### Development Practices

Everything is done through `cargo`, with the most recent edition on the stable
toolchain.

- Test with `cargo test`. Use `--all` to get the whole workspace. See the
  `testing-plan.md` file for more information.
- Build documentation with `cargo doc`. Use `--open` to browse the documentation.
- Format with `cargo fmt` and lint with `cargo clippy`.
- Eventually, we'll benchmark with `cargo bench`.

## Versioning

We'll try to adhere to [semver][] to the extent that makes sense for a language,
once there's enough here to meaningfully 'release'. Eventually, See
`CHANGELOG.md` for more detail.

[semver]: https://semver.org/spec/v2.0.0.html

## License

This project is under the [MIT](https://choosealicense.com/licenses/mit/)
license. See the included `LICENSE` file.
