use dioxus::prelude::*;

const STYLE: Asset = asset!("./style.css");

/// A sidebar navigation menu.
///
/// Renders a titled, scrollable list of labelled items.  The currently active
/// item is highlighted.  The component is fully decoupled from any application
/// context — callers supply the items and a shared [`Signal`] that both drives
/// the highlight and is updated whenever the user clicks a row.
///
/// # Example
///
/// ```rust
/// let selected = use_signal(|| 0_usize);
///
/// Menu {
///     title: "My App",
///     items: vec!["Overview".into(), "Settings".into()],
///     selected,
/// }
/// ```
#[component]
pub fn Menu(
	/// Text shown in the menu header.
	#[props(into)]
	title: String,
	/// Ordered list of item labels to display.
	items: Vec<String>,
	/// Shared signal holding the 0-based index of the selected item.
	/// The menu writes to this signal when the user clicks a row.
	selected: Signal<usize>,
) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }

		nav {
			class: "menu",
			header { "{title}" }
			div {
				for (index, label) in items.into_iter().enumerate() {
					MenuItem {
						key: "{index}",
						index,
						label,
						selected,
					}
				}
			}
		}
	}
}

/// A single row inside a [`Menu`].
#[component]
fn MenuItem(
	/// Position of this item in the parent list.
	index: usize,
	/// Display label.
	label: String,
	/// Shared signal forwarded from the parent [`Menu`].
	/// Read to determine the active state; written on click.
	selected: Signal<usize>,
) -> Element {
	let is_selected = selected() == index;

	rsx! {
		div {
			class: "menu-item",
			class: if is_selected { "selected" },
			onclick: move |_| selected.set(index),
			"{label}"
		}
	}
}
