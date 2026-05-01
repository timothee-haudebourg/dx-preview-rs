# dx-preview

<!-- cargo-rdme start -->

A component preview and development tool for [Dioxus](https://dioxuslabs.com/).

Annotate your components with [`preview`] and get an interactive preview UI
where each property can be inspected and edited live — without writing any
registration code or maintaining a central list.

### Features

- **Zero-config discovery** — components register themselves at link time; no
  central list to maintain.
- **Isolated previews** — each component renders in its own browsing context so
  styles never leak between the preview shell and the component under inspection.
- **Live property editing** — scalar properties (`bool`, `String`, integers) are
  reflected automatically and exposed as interactive controls.
- **Reactive signals** — a `Signal<T>` property is shared between the shell and
  the preview iframe; if the component writes to the signal, the properties panel
  reflects the new value in real time.

### Usage

#### 1. Add the dependency

```toml
[dependencies]
dx-preview = { path = "…" }

[features]
preview = ["dx-preview/web"]
```

#### 2. Annotate your components

Place `#[dx_preview::preview]` above `#[component]`. Use `#[preview(…)]` on
individual parameters to control how they appear in the UI.

```rust
#[dx_preview::preview]
#[component]
pub fn PrimaryButton(
    // Visible and editable in the sidebar.
    #[preview(default = "Click me".to_string())]
    label: String,

    // Not reflectable — hidden from the UI, default value used in the preview.
    #[preview(hide)]
    onclick: EventHandler<MouseEvent>,

    // Hidden with an explicit value rendered inside the preview.
    #[preview(hide, default = rsx! { span { "→" } })]
    children: Element,
) -> Element {
    rsx! { button { onclick, "{label}" } }
}
```

##### Parameter attributes

| Attribute | Effect |
|---|---|
| *(none)* | Visible; type must implement [`model::Reflect`]; default via `Default::default()` |
| `#[preview(default = expr)]` | Visible; use `expr` as the initial value |
| `#[preview(hide)]` | Hidden; `Default::default()` is used in the preview |
| `#[preview(hide, default = expr)]` | Hidden; `expr` is used in the preview |

If a type does not implement [`model::Reflect`] and is not marked
`#[preview(hide)]`, you will get a compile-time error on the offending parameter.

#### 3. Implement `Reflect` for custom types (optional)

`bool`, `String`, `Option<T>`, and all primitive integer types implement
[`model::Reflect`] out of the box. For unit enums, derive it:

```rust
#[derive(dx_preview::Reflect)]
enum Size { Small, Medium, Large }
```

For other custom types, implement the trait manually:

```rust
use dx_preview::model::{Reflect, IntType, Type, Value, TypeError};

struct Radius(u32);

impl Reflect for Radius {
    const TYPE: Type = Type::Int(IntType::U32);

    fn to_value(self) -> Value {
        Value::Int(dx_preview::model::IntValue::U32(self.0))
    }

    fn try_from_value(v: Value) -> Result<Self, TypeError> {
        match v {
            Value::Int(dx_preview::model::IntValue::U32(n)) => Ok(Radius(n)),
            _ => Err(TypeError::InvalidType),
        }
    }
}
```

#### 4. Launch the preview binary

Add a binary target guarded by the `preview` feature so it never appears in
production builds:

```toml
[[bin]]
name = "preview"
path = "src/bin/preview.rs"
required-features = ["preview"]
```

```rust
use dioxus::prelude::*;

extern crate my_components; // ensures components are registered

const TAILWIND: Asset = asset!("/assets/tailwind.css");

fn main() {
    dx_preview::launch(
        dx_preview::Config::default().with_css(TAILWIND),
    );
}
```

Run it with the Dioxus CLI:

```sh
dx serve --features preview --bin preview
```

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

 ### Contribution
 
 Unless you explicitly state otherwise, any contribution intentionally submitted
 for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
 additional terms or conditions.

 Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file before submitting any contribution.