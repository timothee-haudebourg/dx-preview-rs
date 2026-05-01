use dioxus::prelude::*;

const STYLE: Asset = asset!("./style.css");

/// The interactive core of any button-like element.
///
/// Renders the appropriate HTML element based on the supplied action props:
///
/// | Props supplied | Rendered element |
/// |---|---|
/// | `to` | Dioxus [`Link`] (router navigation) |
/// | `onclick` | `<button>` |
/// | neither | Inert `<div>` |
///
/// The `btn-content` CSS class is always applied, which resets
/// browser-default button and anchor styles so that callers start from a
/// clean baseline.  All visual styling (colours, padding, hover effects) is
/// left to the caller — typically via a parent selector in CSS, e.g.
/// `.dropdown-item > .btn-content`.
///
/// When `enabled` is `false` the shared `disabled` semantic class is added
/// and, for `<button>` elements, the native `disabled` attribute is set.
#[component]
pub fn ButtonContent(
	/// Router navigation target (renders as a Dioxus [`Link`]).
	#[props(optional)]
	to: Option<NavigationTarget>,

	/// Click handler (renders as `<button>`).
	#[props(optional)]
	onclick: Option<Callback<MouseEvent>>,

	/// Whether the element is interactive.
	///
	/// When `false` the `disabled` semantic class is applied and, for
	/// `<button>` elements, the native `disabled` attribute is set.
	enabled: bool,

	children: Element,
) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }
		if let Some(target) = to {
			Link {
				class: "dxp-btn-content",
				"data-disabled": if !enabled { "true" },
				to: target,
				{children}
			}
		} else if let Some(cb) = onclick {
			button {
				class: "dxp-btn-content",
				"data-disabled": if !enabled { "true" },
				disabled: !enabled,
				onclick: move |e| cb.call(e),
				{children}
			}
		} else {
			div {
				class: "dxp-btn-content",
				"data-disabled": if !enabled { "true" },
				{children}
			}
		}
	}
}
