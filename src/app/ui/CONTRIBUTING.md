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
├── menu.rs
├── menu.css        ← styles for Menu
└── input/
    ├── bool.rs
    ├── mod.css   ← styles shared by all input components
    └── …
```

```rust
// In src/app/ui/menu.rs
const STYLE: Asset = asset!("src/app/ui/menu.css");
```

Do not put component styles under `assets/`.  That directory is reserved for
truly global or third-party assets (e.g. a Tailwind bundle, external fonts)
that are not owned by a single component.

---

## CSS

### Class names

Use **kebab-case** for every class name.

```css
/* ✓ */
.my-component { … }
.input-bool { … }

/* ✗ */
.my_component { … }
.inputBool { … }
```

### Prefer element selectors over redundant classes

When an HTML element is the only occurrence of its type within the component's
root class, use a **descendant or child selector** instead of adding a
dedicated class to the element.

```css
/* ✓ — the header element is unambiguous inside .menu */
.menu header { … }
.properties-panel > div { … }

/* ✗ — the class carries no information the element type doesn't already give */
.menu__header { … }
.properties-panel__list { … }
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

### Shared semantic classes

State modifiers that carry the same meaning in multiple components are defined
once and reused.  Apply them alongside the component-scoped base class; never
encode them into the base class name.

| Class | Meaning |
|---|---|
| `selected` | The item is the active selection |
| `enabled` | The item / field is active |
| `disabled` | The item / field is inactive / greyed out |
| `empty` | A container currently holds no content |
| `loading` | Content is being fetched or initialised |
| `ready` | Content has finished loading |

Scope their visual effect to the component that owns the context:

```css
/* ✓ — effect is scoped; the shared class name is just a semantic marker */
.menu-item.selected   { background: var(--ui-color-primary); }
.property-editor label.disabled { color: var(--ui-color-text-disabled); }

/* ✗ — global visual rule leaks across components */
.selected { background: blue; }
```

---

## Dioxus RSX

### Class composition

Use **multiple `class:` attributes** on the same element to apply a conditional
modifier.  Do not build the full class string in a `let` binding.

```rust
// ✓
div {
    class: "menu-item",
    class: if is_selected { "selected" },
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
