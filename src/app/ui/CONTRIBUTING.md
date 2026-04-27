# UI Component Authoring Guidelines

Guidelines for writing components in this module.  They are intentionally
framed in general terms so they remain applicable as the component library
grows.

---

## Assets

### Co-locate CSS with its component

Each component's CSS file lives in the same directory as the `.rs` file that
owns it.  Reference it with a path relative to the package root:

```
src/app/ui/
└── menu/
    ├── mod.rs
    └── style.css ← styles for Menu
```

```rust
// In src/app/ui/menu/mod.rs
const STYLE: Asset = asset!("./style.css");
```

Do not put component styles under `assets/`.  That directory is reserved for
truly global or third-party assets (e.g. a Tailwind bundle, external fonts)
that are not owned by a single component.

---

## CSS

### Class names

Use **kebab-case** for every class name.  Every class name must be prefixed
with `dxp-` to avoid collisions with third-party stylesheets.

```css
/* ✓ */
.dxp-my-component { … }
.dxp-input-bool { … }

/* ✗ */
.dxp-my_component { … }
.dxp-inputBool { … }
```

### Prefer element selectors over redundant classes

When an HTML element is the only occurrence of its type within the component's
root class, use a **descendant or child selector** instead of adding a
dedicated class to the element.

```css
/* ✓ — the header element is unambiguous inside .menu */
.menu header { … }
.panel > div { … }

/* ✗ — the class carries no information the element type doesn't already give */
.menu__header { … }
.panel__list { … }
```

Reserve explicit classes for elements that cannot be distinguished by type or
structural position alone (e.g. two sibling `<div>`s that need independent
styles, or a deeply nested element that would require a fragile path selector).

When position is meaningful and stable, `:first-child` / `:last-child` /
`:nth-child()` are acceptable:

```css
/* ✓ — two <span>s with distinct roles, position is part of the contract */
.empty-view :first-child { font-size: 2rem; }
.empty-view :last-child  { font-size: 0.9rem; }
```

### State data attributes

Runtime state is encoded in `data-*` attributes instead of classes.  This
keeps identity classes — which are always present on an element — separate from
state that changes at runtime, and avoids accidental collisions with utility
CSS frameworks.

**Naming convention**

| Kind | Form | Example |
|---|---|---|
| Boolean | bare attribute | `data-disabled`, `data-selected` |
| Enum / multi-value | attribute with value | `data-state="loading"`, `data-role="destructive"` |

**CSS selector pattern** — always scoped under the component's root class:

```css
/* ✓ */
.dxp-menu-item[data-selected] { background: var(--ui-color-primary); }
.dxp-dropdown-item[data-role="destructive"] { color: var(--ui-color-danger); }
```

**RSX pattern** — when the `if` branch is not taken the attribute is omitted
entirely, identical behaviour to an untaken `class:` branch:

```rust
// ✓
div {
    class: "dxp-menu-item",
    "data-selected": if is_selected { "true" },
}
```

Visual state must never be encoded in identity class names — no
`dxp-menu-item--selected`, `dxp-menu-item--disabled`, etc.

---

## Dioxus RSX

### Class composition

Use **multiple `class:` attributes** on the same element to apply a conditional
modifier.  Do not build the full class string in a `let` binding.

```rust
// ✓
div {
    class: "dxp-menu-item",
    "data-selected": if is_selected { "true" },
}

// ✗
let class = if is_selected { "menu-item selected" } else { "menu-item" };
div { class }
```

A `class:` attribute whose expression evaluates to an empty string or whose
`if` branch is not taken contributes nothing to the rendered output — there is
no need for a matching `else ""` branch.

Multiple `class:` values are joined with a single space in the order they
appear.

### Prefer signals to callbacks

When a component needs to expose a piece of mutable state to its caller, accept
a `Signal<T>` rather than a value plus an `EventHandler`:

```rust
// ✓
#[component]
fn Menu(selected: Signal<usize>, …) -> Element { … }

// ✗
#[component]
fn Menu(selected: usize, on_select: EventHandler<usize>, …) -> Element { … }
```

The signal can be read and written directly inside the component, the caller
retains full reactivity, and no forwarding closure is needed.