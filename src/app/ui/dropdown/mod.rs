use dioxus::prelude::*;

use super::{ButtonContent, Role};

const STYLE: Asset = asset!("./style.css");

/// Height of one rendered menu item in rem, used to compute `max-height`
/// when a maximum visible count is requested.
///
/// Accounts for the item's own padding plus the inner content height at the
/// default font size.
const MENU_ITEM_HEIGHT_REM: f32 = 2.5;

/// A styled dropdown menu container.
///
/// Renders as a `<ul role="menu">` and is designed to be placed inside a
/// [`crate::Popup`]. Provide [`DropdownMenuItem`] components as children.
///
/// # Example
///
/// ```no_run
/// rsx! {
///     Popup { show, trigger,
///         DropdownMenu { labeled_by: "trigger-id",
///             DropdownMenuItem { onclick: |_| {}, "Edit" }
///             DropdownMenuItem { onclick: |_| {}, role: Role::Destructive, "Delete" }
///         }
///     }
/// }
/// ```
#[component]
pub fn DropdownMenu(
	/// Maximum number of visible items before the list scrolls.
	/// When omitted the menu grows to fit all items.
	#[props(optional)]
	max_height: Option<f32>,

	/// `id` of the element that triggers this menu (sets `aria-labelledby`).
	#[props(optional)]
	labeled_by: Option<String>,

	/// Accessible label when no triggering element exists (sets `aria-label`).
	#[props(optional)]
	label: Option<String>,

	children: Element,
) -> Element {
	let max_height_style =
		max_height.map(|h| format!("max-height: {}rem;", h * MENU_ITEM_HEIGHT_REM));

	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }
		ul {
			role: "menu",
			aria_labelledby: labeled_by,
			aria_label: label,
			class: "dxp-dropdown-menu",
			style: max_height_style,
			{children}
		}
	}
}

/// A single item inside a [`DropdownMenu`].
///
/// Renders as a `<li role="menuitem">` wrapping a [`ButtonContent`], which
/// selects the appropriate interactive element based on the supplied action
/// props:
///
/// | Props supplied | Inner element |
/// |---|---|
/// | `to` | Dioxus `Link` (router navigation) |
/// | `onclick` | `<button>` |
/// | neither | Inert `<div>` |
///
/// Exactly one of `to` or `onclick` should be provided; if neither is given
/// the item is rendered as a non-interactive row.
///
/// Apply `role: Role::Destructive` for actions such as deletion; this
/// colours the label red and applies a red hover background.
#[component]
pub fn DropdownMenuItem(
	/// Router navigation target (renders inner element as a Dioxus `Link`).
	#[props(optional)]
	to: Option<NavigationTarget>,

	/// Click handler (renders inner element as `<button>`).
	#[props(optional)]
	onclick: Option<Callback<MouseEvent>>,

	/// Visual role of the item. Defaults to [`Role::Normal`].
	#[props(optional)]
	role: Role,

	/// Whether the item is interactive. Defaults to `true`.
	#[props(default = true)]
	enabled: bool,

	children: Element,
) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }
		li {
			role: "menuitem",
			class: "dxp-dropdown-item",
			"data-role": if role == Role::Destructive { "destructive" },
			ButtonContent { to, onclick, enabled, {children} }
		}
	}
}
