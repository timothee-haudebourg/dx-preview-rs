use std::{
	hash::{Hash, Hasher},
	marker::PhantomData,
	rc::Rc,
};

use dioxus::{
	core::{consume_context, provide_context, try_consume_context, use_hook_with_cleanup},
	hooks::{use_context, use_effect, use_signal},
	prelude::{ReadableRef, WritableRef},
	signals::{ReadableExt, Signal, WritableExt},
};

use crate::model::Reflect;

use super::{Value, ValueSource};

mod context;

use context::*;

/// Provide a [`ValueContext`] for `side` and register `on_write` as the
/// callback to invoke whenever a [`ReactiveValue`] is written to locally.
///
/// The protocol passes a closure here that forwards updates to the remote
/// window. Must be called from a component/hook scope.
pub fn provide_signal_value_context(
	side: ValueSource,
	on_write: impl 'static + Fn(ReactiveValueId, &Value),
) {
	provide_context(Rc::new(ValueContext::new(side, on_write)));
}

/// Uniquely identifies a [`ReactiveValue`] across both windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ReactiveValueId {
	/// Which window created (owns) this signal.
	pub owner: ValueSource,
	/// Unique index within the owning window's [`Slab`] allocation.
	pub id: usize,
}

/// A reactive property value whose underlying signal is shared between windows.
///
/// The wire format is `{ id, value }`: the `value` snapshot lets the receiving
/// window register the signal on first sight. The field does not appear on the
/// Rust struct — it is looked up from the [`ValueContext`] when serialising and
/// consumed during deserialization.
///
/// Equality and hashing compare only `id`.
#[derive(Debug, Clone, Copy)]
pub struct ReactiveValue {
	pub id: ReactiveValueId,
}

impl PartialEq for ReactiveValue {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for ReactiveValue {}

impl Hash for ReactiveValue {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

// Custom Serialize: bundle the current value from the context.
impl serde::Serialize for ReactiveValue {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(serde::Serialize)]
		struct Wire<'a> {
			id: ReactiveValueId,
			value: &'a Value,
		}
		let value = consume_context::<Rc<ValueContext>>()
			.current_value(self.id)
			.unwrap_or(Value::Option(None));
		Wire {
			id: self.id,
			value: &value,
		}
		.serialize(serializer)
	}
}

// Custom Deserialize: extract the bundled value, register the signal if needed,
// and return a bare handle.
impl<'de> serde::Deserialize<'de> for ReactiveValue {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(serde::Deserialize)]
		struct Wire {
			id: ReactiveValueId,
			value: Value,
		}
		let Wire { id, value } = Wire::deserialize(deserializer)?;
		try_consume_context::<Rc<ValueContext>>()
			.ok_or_else(|| {
				serde::de::Error::custom(
					"ReactiveValue cannot be deserialized outside a ValueContext",
				)
			})?
			.apply(id, value);
		Ok(ReactiveValue { id })
	}
}

/// Read reference to the value of a [`ReactiveValue`].
///
/// Subscribes the calling scope to future changes, exactly like [`Signal::read`].
/// Derefs to [`Value`]. The lifetime `'a` is tied to the originating
/// [`ReactiveValue`] borrow.
pub type ReactiveReadRef<'a> = ReadableRef<'a, Signal<Value>>;

/// Write reference to the value of a [`ReactiveValue`].
///
/// Provides mutable access to the underlying [`Value`]. On drop:
/// 1. Fires the [`ValueContext`]'s `on_write` callback (remote notification).
/// 2. The inner write guard drops, notifying Dioxus subscribers (local UI).
///
/// The lifetime `'a` is tied to the originating [`ReactiveValue`] borrow.
pub struct ReactiveWriteRef<'a> {
	guard: WritableRef<'static, Signal<Value>>,
	id: ReactiveValueId,
	ctx: Rc<ValueContext>,
	_phantom: PhantomData<&'a ReactiveValue>,
}

impl std::ops::Deref for ReactiveWriteRef<'_> {
	type Target = Value;

	fn deref(&self) -> &Value {
		&self.guard
	}
}

impl std::ops::DerefMut for ReactiveWriteRef<'_> {
	fn deref_mut(&mut self) -> &mut Value {
		&mut self.guard
	}
}

impl Drop for ReactiveWriteRef<'_> {
	fn drop(&mut self) {
		self.ctx.notify_write(self.id, &self.guard);
		// `self.guard` drops after this body returns, notifying Dioxus subscribers.
	}
}

/// Create a new [`ReactiveValue`] with component lifecycle management.
///
/// Like [`ReactiveValue::new`] but holds a reference for as long as the
/// calling component is mounted; the signal entry is freed on unmount.
pub fn use_reactive_value(initial: impl FnOnce() -> Value) -> ReactiveValue {
	let ctx: Rc<ValueContext> = use_context();
	let ctx_cleanup = ctx.clone();

	use_hook_with_cleanup(
		move || {
			let rv = ctx.create(initial());
			ctx.inc_ref(rv.id);
			rv
		},
		move |rv| ctx_cleanup.dec_ref(rv.id),
	)
}

impl ReactiveValue {
	/// The inverse of [`use_adapter`](ReactiveValue::use_adapter).
	///
	/// Creates a [`ReactiveValue`] backed by an existing `Signal<T>` and
	/// wires two effects:
	/// - **`Signal<T>` → `ReactiveValue`**: local writes to the signal are
	///   forwarded to the remote window via the `on_write` callback.
	/// - **`ReactiveValue` → `Signal<T>`**: remote writes update the local
	///   signal (equality-checked to break feedback loops).
	///
	/// Must be called from a Dioxus hook context.
	pub fn use_from_signal<T: Reflect + Clone + PartialEq + 'static>(signal: Signal<T>) -> Self {
		let rv = use_reactive_value(|| signal.peek().clone().to_value());

		// Signal<T> → ReactiveValue: propagates local changes to the remote.
		use_effect(move || {
			*rv.write() = signal.read().clone().to_value();
		});

		// ReactiveValue → Signal<T>: applies remote changes to the local signal.
		use_effect(move || {
			if let Ok(t) = T::try_from_value((*rv.read()).clone()) {
				if *signal.peek() != t {
					let mut signal = signal;
					signal.set(t);
				}
			}
		});

		rv
	}

	/// Create a new reactive value backed by `initial`.
	///
	/// The signal is allocated in the [`ValueContext`]'s scope and lives until
	/// that scope drops. For lifecycle management tied to a specific component,
	/// use [`use_reactive_value`] instead.
	pub fn new(initial: Value) -> Self {
		consume_context::<Rc<ValueContext>>().create(initial)
	}

	/// Get a read reference to the current value.
	///
	/// Subscribes the calling scope to future changes. Panics if no
	/// [`ValueContext`] is active or the signal has been freed.
	pub fn read(&self) -> ReactiveReadRef<'_> {
		let ctx = consume_context::<Rc<ValueContext>>();
		let signal = ctx
			.get_signal(self.id)
			.expect("ReactiveValue::read: signal not found in ValueContext");
		signal.read_unchecked()
	}

	/// Apply a value without notifying the remote window.
	///
	/// Delegates to [`ValueContext::apply`]: creates the signal if it is a new
	/// remote signal, or updates the stored value otherwise. Used when
	/// receiving a [`WriteSignal`](crate::app::protocol) message to avoid
	/// echoing the notification back to the sender.
	pub fn write_silent(&self, value: Value) {
		consume_context::<Rc<ValueContext>>().apply(self.id, value);
	}

	/// Get a write reference to the current value.
	///
	/// The remote window is notified when this reference is dropped and the
	/// value has changed. Panics if no [`ValueContext`] is active or the
	/// signal has been freed.
	pub fn write(&self) -> ReactiveWriteRef<'_> {
		let ctx = consume_context::<Rc<ValueContext>>();
		let signal = ctx
			.get_signal(self.id)
			.expect("ReactiveValue::write: signal not found in ValueContext");
		let guard = signal.write_unchecked();
		ReactiveWriteRef {
			guard,
			id: self.id,
			ctx,
			_phantom: PhantomData,
		}
	}

	/// Returns a [`Signal<T>`] that stays in sync with this reactive value.
	///
	/// - Changes arriving from the remote flow through `try_into` into the
	///   returned signal.
	/// - Local writes to the returned signal are converted via `from` and
	///   forwarded to the remote window through [`ValueContext`]'s `on_write`
	///   callback (via [`ReactiveWriteRef`]'s [`Drop`] impl).
	///
	/// Feedback loops are prevented by the equality check inside
	/// [`ReactiveWriteRef::drop`] and by only updating `adapted` when its
	/// value would actually change.
	pub fn use_adapter<T: Reflect + Clone + PartialEq + 'static>(&self) -> Signal<T> {
		let initial = T::try_from_value((*self.read()).clone())
			.ok()
			.expect("initial adapter conversion must succeed");
		let mut adapted = use_signal(move || initial);

		let this = *self;

		// Underlying → adapted.
		use_effect(move || {
			let underlying = this.read(); // ReactiveReadRef, subscribes to signal changes
			if let Ok(new_adapted) = T::try_from_value((*underlying).clone()) {
				if *adapted.peek() != new_adapted {
					adapted.set(new_adapted);
				}
			}
		});

		// Adapted → underlying (with write notification via ReactiveWriteRef::drop).
		use_effect(move || {
			*this.write() = adapted.read().clone().to_value();
		});

		adapted
	}
}
