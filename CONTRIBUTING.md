# How to Contribute

Contributions are welcome! Here is how to contribute.

## Contribution Guidelines

Please read the [contribution guidelines](https://github.com/timothee-haudebourg/admin/blob/main/files/CONTRIBUTING.md) before submitting your changes. Contributions that do not adhere to these guidelines may be rejected without further explanation.

## Architecture

The project is split into two crates:

- **`dx-preview-macro`** — a proc-macro crate providing the `#[preview]` attribute.
- **`dx-preview`** — the runtime library, with an optional `web` feature that enables the
  interactive browser UI.

Refer to the crate and module documentation for implementation details.

## Previewing UI components

The UI components that make up the dx-preview shell (inputs, menus, etc.) are
themselves annotated with `#[preview]` and can be inspected interactively. To
launch the preview shell against the library's own components, run:

```sh
dx serve --features preview
```

This is the recommended way to develop and test changes to the shell's UI
components.

## README.md

The `README.md` is generated from the crate-level doc comment in `src/lib.rs` using
[`cargo rdme`](https://github.com/orium/cargo-rdme). After editing `src/lib.rs`, run:

```sh
cargo rdme
```

Do not edit `README.md` directly between the `<!-- cargo-rdme start -->` and
`<!-- cargo-rdme end -->` markers — those changes will be overwritten.