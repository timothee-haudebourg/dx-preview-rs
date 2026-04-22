use std::rc::Rc;

use dioxus::{
	core::use_hook_with_cleanup,
	hooks::use_signal,
	signals::{Signal, WritableExt},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{HtmlIFrameElement, MessageEvent, Window};

use crate::model::{
	ReactiveValue, ReactiveValueId, Value, ValueSource, provide_signal_value_context,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ParentMessage {
	Set(usize, Value),
	/// Parent wrote a new value to a reactive signal.
	WriteSignal {
		id: ReactiveValueId,
		value: Value,
	},
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ChildMessage {
	Ready,
	Set(usize, Value),
	/// Child wrote a new value to a reactive signal.
	WriteSignal {
		id: ReactiveValueId,
		value: Value,
	},
}

pub fn use_child(initial_values: impl FnOnce() -> Vec<Value>) -> Signal<Vec<Value>> {
	provide_signal_value_context(ValueSource::Child, |id, value| {
		if let Some(parent) = parent_window() {
			post_message(
				&parent,
				ChildMessage::WriteSignal {
					id,
					value: value.clone(),
				},
			);
		}
	});

	let mut values = use_signal(initial_values);

	use_message({
		move |message| match message {
			ParentMessage::Set(i, value) => {
				let mut values = values.write();
				if let Some(s) = values.get_mut(i) {
					*s = value;
				}
			}
			ParentMessage::WriteSignal { id, value } => {
				ReactiveValue { id }.write_silent(value);
			}
		}
	});

	values
}

pub fn use_parent() -> Signal<bool> {
	provide_signal_value_context(ValueSource::Parent, |id, value| {
		if let Some(child) = child_window() {
			post_message(
				&child,
				ParentMessage::WriteSignal {
					id,
					value: value.clone(),
				},
			);
		}
	});

	let mut ready = use_signal(|| false);

	use_message({
		move |message| match message {
			ChildMessage::Ready => ready.set(true),
			ChildMessage::Set(_, _) => {
				// TODO: propagate child-initiated property value updates to the parent
			}
			ChildMessage::WriteSignal { id, value } => {
				ReactiveValue { id }.write_silent(value);
			}
		}
	});

	ready
}

fn use_message<T>(mut f: impl 'static + FnMut(T))
where
	T: DeserializeOwned,
{
	use_hook_with_cleanup(
		move || {
			let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
				if let Some(json) = e.data().as_string()
					&& let Ok(t) = serde_json::from_str::<T>(&json)
				{
					f(t)
				}
			});

			if let Some(w) = web_sys::window() {
				let _ =
					w.add_event_listener_with_callback("message", closure.as_ref().unchecked_ref());
			}

			Rc::new(closure)
		},
		|closure| {
			if let Some(w) = web_sys::window() {
				let _ = w.remove_event_listener_with_callback(
					"message",
					(*closure).as_ref().unchecked_ref(),
				);
			}
		},
	);
}

fn parent_window() -> Option<Window> {
	web_sys::window()?.parent().ok()?
}

fn post_message<T>(window: &Window, message: T)
where
	T: Serialize,
{
	let Ok(json) = serde_json::to_string(&message) else {
		return;
	};

	let _ = window.post_message(&JsValue::from_str(&json), "*");
}

/// Sends [`Message::Ready`] to the parent window, signalling that this iframe
/// has rendered and is ready to receive value updates.
pub fn notify_child_ready() {
	let Some(parent) = parent_window() else {
		return;
	};

	post_message(&parent, ChildMessage::Ready);
}

/// The `id` attribute set on the preview iframe element.
pub const CHILD_ID: &str = "dx-preview";

fn child_window() -> Option<Window> {
	web_sys::window()?
		.document()?
		.get_element_by_id(CHILD_ID)?
		.dyn_into::<HtmlIFrameElement>()
		.ok()?
		.content_window()
}

pub fn set_child_value(i: usize, value: Value) {
	let Some(child) = child_window() else { return };

	post_message(&child, ParentMessage::Set(i, value));
}

pub fn set_parent_value(i: usize, value: Value) {
	let Some(child) = parent_window() else { return };

	post_message(&child, ChildMessage::Set(i, value));
}
