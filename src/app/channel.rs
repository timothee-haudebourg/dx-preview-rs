//! Shell ↔ iframe communication channel.
//!
//! The shell and the isolation iframe communicate via `postMessage` so the WASM
//! module is never reloaded between property edits. The protocol uses two message
//! types:
//!
//! - [`Message::Values`] — sent by the shell whenever the user edits a property.
//! - [`Message::Ready`] — sent by the iframe after its first render, signalling
//!   that WASM has initialised and CSS has been applied.
//!
//! The shell keeps the iframe hidden (opacity 0) until `Ready` arrives, which
//! eliminates flash-of-unstyled-content on component selection.

use std::rc::Rc;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{HtmlIFrameElement, MessageEvent};

use crate::model::Value;

type Values = Vec<Option<Value>>;

// ── Protocol ──────────────────────────────────────────────────────────────────

/// Messages exchanged between the shell and the preview iframe.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum Message {
	/// Parent → iframe: set the component's property values.
	Values(Values),
	/// Iframe → parent: WASM has rendered and CSS has been applied.
	Ready,
}

// ── RAII listener ─────────────────────────────────────────────────────────────

/// Registers a `message` event listener on `window` and removes it on drop.
struct Listener {
	closure: Closure<dyn FnMut(MessageEvent)>,
}

impl Listener {
	fn new(closure: Closure<dyn FnMut(MessageEvent)>) -> Self {
		if let Some(w) = web_sys::window() {
			let _ = w.add_event_listener_with_callback("message", closure.as_ref().unchecked_ref());
		}
		Self { closure }
	}
}

impl Drop for Listener {
	fn drop(&mut self) {
		if let Some(w) = web_sys::window() {
			let _ = w.remove_event_listener_with_callback(
				"message",
				self.closure.as_ref().unchecked_ref(),
			);
		}
	}
}

// ── Sender (parent side) ──────────────────────────────────────────────────────

/// The `id` attribute set on the preview iframe element.
pub const IFRAME_ID: &str = "dx-preview";

/// Sends typed messages to the preview iframe's content window.
///
/// Locates the iframe by [`IFRAME_ID`] on each call so no DOM reference needs
/// to be stored.
#[derive(Clone, Copy)]
pub struct IframeSender;

impl IframeSender {
	/// Send the current property values to the iframe.
	pub fn send_values(self, values: &[Option<Value>]) {
		self.post(Message::Values(values.to_vec()));
	}

	fn post(self, msg: Message) {
		if let Ok(json) = serde_json::to_string(&msg)
			&& let Some(cw) = Self::content_window()
		{
			let _ = cw.post_message(&JsValue::from_str(&json), "*");
		}
	}

	fn content_window() -> Option<web_sys::Window> {
		web_sys::window()?
			.document()?
			.get_element_by_id(IFRAME_ID)?
			.dyn_into::<HtmlIFrameElement>()
			.ok()?
			.content_window()
	}
}

/// Hook: returns a `Signal<bool>` that becomes `true` once the iframe sends its
/// [`Message::Ready`] notification.
///
/// The underlying event listener is automatically removed when the component
/// unmounts.
pub fn use_iframe_ready() -> Signal<bool> {
	let mut ready = use_signal(|| false);

	let _listener = use_hook(|| {
		Rc::new(Listener::new(Closure::<dyn FnMut(MessageEvent)>::new(
			move |e: MessageEvent| {
				if let Some(json) = e.data().as_string()
					&& matches!(serde_json::from_str::<Message>(&json), Ok(Message::Ready))
				{
					ready.set(true);
				}
			},
		)))
	});

	ready
}

// ── Receiver (iframe side) ────────────────────────────────────────────────────

/// Sends [`Message::Ready`] to the parent window, signalling that this iframe
/// has rendered and is ready to receive value updates.
pub fn notify_parent_ready() {
	if let Ok(json) = serde_json::to_string(&Message::Ready)
		&& let Some(parent) = web_sys::window()
			.as_ref()
			.and_then(|w| w.parent().ok())
			.flatten()
	{
		let _ = parent.post_message(&JsValue::from_str(&json), "*");
	}
}

/// Hook: listens for [`Message::Values`] sent by the parent frame and exposes
/// them as a reactive `Signal<Values>`.
///
/// `defaults` is used as the initial value until the parent sends the first
/// update. The listener is removed automatically on unmount.
pub fn use_incoming_values(defaults: impl FnOnce() -> Values) -> Signal<Values> {
	let mut values = use_signal(defaults);

	let _listener = use_hook(|| {
		Rc::new(Listener::new(Closure::<dyn FnMut(MessageEvent)>::new(
			move |e: MessageEvent| {
				if let Some(json) = e.data().as_string()
					&& let Ok(Message::Values(new_vals)) = serde_json::from_str::<Message>(&json)
				{
					values.set(new_vals);
				}
			},
		)))
	});

	values
}
