use dioxus::prelude::*;

const STYLE: Asset = asset!("./style.css");

/// A styled container for popup content.
///
/// Provides the standard shadow, border, and background used by all floating
/// panels in the design system. Wrap your popup body in this when building
/// custom popups.
#[component]
pub fn PopupFrame(children: Element) -> Element {
	rsx! {
		document::Link { rel: "stylesheet", href: STYLE }
		div { class: "dxp-popup-frame", {children} }
	}
}

/// Horizontal alignment of popup content relative to the anchor element.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "preview", derive(dx_preview_macro::Reflect))]
#[cfg_attr(feature = "preview", preview(crate = crate))]
pub enum PopupAlignment {
	/// Align the popup's left edge with the anchor's left edge.
	Start,

	/// Centre the popup over the anchor.
	#[default]
	Center,

	/// Align the popup's right edge with the anchor's right edge.
	End,
}

impl PopupAlignment {
	fn css_justify(self) -> &'static str {
		match self {
			Self::Start => "flex-start",
			Self::Center => "center",
			Self::End => "flex-end",
		}
	}
}

/// A floating panel anchored below an element, rendered above all other
/// content using `position: fixed`.
///
/// Because it uses `position: fixed` the popup escapes `overflow: hidden` and
/// `overflow: auto` on all ancestor elements — the same guarantee that Leptos
/// achieves via `<Portal>`, without requiring any DOM portal API.
///
/// ## How it works
///
/// The component wraps the `trigger` slot in a thin `display: inline-block`
/// div. When that div mounts, the underlying `web_sys::Element` is extracted
/// via [`MountedData::downcast`] and stored. Each time `show` changes to
/// `true`, [`Element::get_bounding_client_rect`] is called **synchronously**
/// on the stored element, so the popup is always positioned at the current
/// location of the trigger — even after scrolling or layout shifts.
///
/// ## Click-outside
///
/// An invisible full-screen overlay sits beneath the popup at
/// `z-index: 9999`. Clicking anywhere outside the popup sets `show` to
/// `false`.
///
/// ## Example
///
/// ```no_run
/// let show = use_signal(|| false);
/// rsx! {
///     Popup {
///         show,
///         trigger: rsx! {
///             button {
///                 onclick: move |_| show.set(!show()),
///                 "Open menu"
///             }
///         },
///         PopoverFrame { "Popup content" }
///     }
/// }
/// ```
#[component]
pub fn Popup(
	/// The element that the popup is anchored to. Rendered inside an
	/// invisible wrapper that is used to measure the popup's position.
	trigger: Element,

	/// Whether the popup is currently visible.
	///
	/// Passed as a writable signal so the popup can close itself when the
	/// user clicks outside it.
	mut show: Signal<bool>,

	/// Horizontal alignment of the popup content relative to the anchor.
	#[props(default)]
	align: PopupAlignment,

	/// Extra vertical gap in pixels added between the anchor's bottom edge
	/// and the popup's top edge.
	#[props(default)]
	gap: f64,

	/// The popup body. Rendered inside the floating panel.
	children: Element,
) -> Element {
	let mut anchor_el: CopyValue<Option<web_sys::Element>> = use_hook(|| CopyValue::new(None));

	let popup_rect = move || {
		anchor_el.read().as_ref().map(|el| {
			let r = el.get_bounding_client_rect();
			(r.left(), r.bottom() + gap, r.width())
		})
	};

	// Extract the raw web_sys::Element when the wrapper mounts.
	#[cfg(feature = "web")]
	let on_mounted = move |e: Event<MountedData>| {
		anchor_el.set(e.downcast::<web_sys::Element>().cloned());
	};

	#[cfg(not(feature = "web"))]
	let on_mounted = move |_: Event<MountedData>| {};

	let justify = align.css_justify();

	rsx! {
		// Anchor wrapper — invisible in layout, used only for measurement.
		div { style: "display: inline-block;", onmounted: on_mounted, {trigger} }

		if show() {
			if let Some((left, top, width)) = popup_rect() {
				// Invisible full-screen overlay — absorbs clicks outside the popup.
				div {
					style: "position: fixed; inset: 0; z-index: 9999;",
					onclick: move |_| show.set(false),
				}

				// Popup panel — rendered above the overlay.
				div {
					style: "position: fixed; \
							left: {left}px; \
							top: {top}px; \
							width: {width}px; \
							display: flex; \
							justify-content: {justify}; \
							z-index: 10000;",
					// Stop clicks inside the popup from reaching the overlay.
					onclick: move |e| e.stop_propagation(),
					{children}
				}
			}
		}
	}
}
