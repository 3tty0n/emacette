# Contributing to emacette

Thanks for taking the time to contribute.

- **Bug reports and feature requests:** open a GitHub issue. Include the
  emacette version, terminal, and the smallest reproduction you can.
- **Code contributions:** start with the
  [development guide](docs/development/README.md), then read
  [Contributing](docs/development/contributing.md) for the workflow and
  conventions.
- **Documentation:** the user guide lives in `docs/user/`; architecture,
  testing, and release information lives in `docs/development/`.

Before opening a pull request, run:

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
make e2e   # for terminal, rendering, or CLI changes
```

By contributing, you agree that your contribution is licensed under the
[MIT License](LICENSE).
