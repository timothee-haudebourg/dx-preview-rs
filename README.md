# dx-preview

<!-- cargo-rdme start -->

A component preview and development tool for [Dioxus](https://dioxuslabs.com/).

Annotate your components with [`preview`] and get an interactive storybook-style
UI where each property can be inspected and edited live — without writing any
registration code or maintaining a central list.

### Features

- **Zero-config discovery** — components register themselves at link time; no
  central list to maintain.
- **Isolated previews** — each component renders in its own browsing context so
  styles never leak between the preview and the shell UI.
- **Live property editing** — scalar properties (`bool`, `String`, integers) are
  reflected automatically and exposed as interactive controls.

### Usage

#### 1. Add the dependency

```toml
[dependencies]
dx-preview = { path = "…" }

[features]
storybook = ["dx-preview/app"]
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
| *(none)* | Visible; type must implement [`book::ShowcaseType`]; default via `Default::default()` |
| `#[preview(default = expr)]` | Visible; use `expr` as the initial value |
| `#[preview(hide)]` | Hidden; `Default::default()` is used in the preview |
| `#[preview(hide, default = expr)]` | Hidden; `expr` is used in the preview |

If a type does not implement [`book::ShowcaseType`] and is not marked
`#[preview(hide)]`, you will get a compile-time error on the offending parameter.

#### 3. Implement `ShowcaseType` for custom types (optional)

`bool`, `String`, and all primitive integer types are supported out of the box.
For your own types, implement the trait:

```rust
use dx_preview::book::{ShowcaseType, IntType, Type, Value, TypeError};

struct Radius(u32);

impl ShowcaseType for Radius {
    const TYPE: Type = Type::Int(IntType::U32);

    fn to_value(&self) -> Value {
        Value::Int(dx_preview::book::IntValue::U32(self.0))
    }

    fn try_from_value(v: Value) -> Result<Self, TypeError> {
        match v {
            Value::Int(dx_preview::book::IntValue::U32(n)) => Ok(Radius(n)),
            _ => Err(TypeError),
        }
    }
}
```

#### 4. Launch the preview binary

Add a binary target guarded by the `storybook` feature so it never appears in
production builds:

```toml
[[bin]]
name = "storybook"
path = "src/bin/storybook.rs"
required-features = ["storybook"]
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
dx serve --features storybook --bin storybook --platform web
```

<!-- cargo-rdme end -->

## License

Licensed under the [MIT license](LICENSE).
