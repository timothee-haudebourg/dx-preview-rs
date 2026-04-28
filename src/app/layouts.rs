use dioxus::prelude::*;

/// A minimal preview layout that wraps the previewed component in a padded,
/// width-capped container.
///
/// Use this as the `layout` argument to [`preview`](dx_preview_macro::preview)
/// for components whose previews would otherwise stretch to fill the iframe
/// (e.g. form inputs).
///
/// # Example
///
/// ```rust,ignore
/// #[preview(layout = ::dx_preview::PaddedLayout)]
/// #[component]
/// pub fn MyInput(value: Signal<String>) -> Element { ... }
/// ```
#[component]
pub fn PaddedLayout(children: Element) -> Element {
	rsx! {
		div {
			style: "display: flex; align-items: center; justify-content: center; padding: 2rem; width: 100%; height: 100%; max-width: 24rem; box-sizing: border-box; margin: 0 auto;",
			{children}
		}
	}
}
