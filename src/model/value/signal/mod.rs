use std::{
	hash::{Hash, Hasher},
	rc::Rc,
};

use dioxus::{
	core::{
		ScopeId, Subscribers, consume_context, provide_context, try_consume_context,
		use_hook_with_cleanup,
	},
	hooks::use_context,
	prelude::{ReadableRef, WritableRef, WriteLock},
	signals::{BorrowError, BorrowMutError, Readable, Signal, UnsyncStorage, Writable},
};

use crate::model::Scoped;

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

/// Write metadata for [`ReactiveValue`].
///
/// Stored as the `data` field of [`WriteLock`]. Because [`WriteLock`] drops
/// `data` **before** `write` (reverse declaration order), this metadata can
/// safely read the final value through a raw pointer while the write guard is
/// still held, then fire the remote `on_write` notification.
///
/// After `ReactiveWriteMetadata` drops, its `inner_meta` field
/// ([`SignalSubscriberDrop`]) drops, notifying local Dioxus subscribers.
/// Finally the write guard in `WriteLock.write` releases the borrow.
pub struct ReactiveWriteMetadata {
	/// Raw pointer to the locked value. Valid for the duration of this drop
	/// because the write guard outlives this struct.
	value_ptr: *const Value,
	/// Dioxus subscriber-drop metadata from the underlying signal.
	/// Held solely for its [`Drop`] side effect (notifies local subscribers);
	/// never read directly.
	#[allow(dead_code)]
	inner_meta: <Signal<Value> as Writable>::WriteMetadata,
	id: ReactiveValueId,
	ctx: Rc<ValueContext>,
}

impl Drop for ReactiveWriteMetadata {
	fn drop(&mut self) {
		// Safety: `WriteLock` drops `data` (us) before `write` (the write
		// guard). The write guard is therefore still alive and holding the
		// borrow, so `value_ptr` is valid and points to the current value.
		let value = unsafe { &*self.value_ptr };
		self.ctx.notify_write(self.id, value);
		// `self.inner_meta` (SignalSubscriberDrop) drops after this block,
		// notifying Dioxus subscribers.
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
	/// Create a new reactive value backed by `initial`.
	///
	/// The signal is allocated in the [`ValueContext`]'s scope and lives until
	/// that scope drops. For lifecycle management tied to a specific component,
	/// use [`use_reactive_value`] instead.
	pub fn new(initial: Value) -> Self {
		consume_context::<Rc<ValueContext>>().create(initial)
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
}

/// Retrieve the underlying [`Signal<Value>`] for a [`ReactiveValue`], panicking
/// if the context is missing or the signal has been freed.
fn get_signal(rv: &ReactiveValue) -> Signal<Value> {
	consume_context::<Rc<ValueContext>>()
		.get_signal(rv.id)
		.expect("ReactiveValue: signal not found in ValueContext")
}

impl Scoped for ReactiveValue {
	fn scope(&self) -> ScopeId {
		// The signal is created in the ValueContext's scope; its origin scope
		// is therefore the context's own scope.
		get_signal(self).origin_scope()
	}
}

impl Readable for ReactiveValue {
	type Target = Value;
	type Storage = UnsyncStorage;

	fn try_read_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
		get_signal(self).try_read_unchecked()
	}

	fn try_peek_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
		get_signal(self).try_peek_unchecked()
	}

	fn subscribers(&self) -> Subscribers {
		get_signal(self).subscribers()
	}
}

impl Writable for ReactiveValue {
	type WriteMetadata = ReactiveWriteMetadata;

	fn try_write_unchecked(&self) -> Result<WritableRef<'static, Self>, BorrowMutError>
	where
		Self::Target: 'static,
	{
		let ctx = consume_context::<Rc<ValueContext>>();
		let write = get_signal(self).try_write_unchecked()?;
		// Get a pointer to the value while the write guard is held.
		let value_ptr: *const Value = &*write as *const Value;
		let (inner_write, inner_meta) = write.into_parts();
		Ok(WriteLock::new_with_metadata(
			inner_write,
			ReactiveWriteMetadata {
				value_ptr,
				inner_meta,
				id: self.id,
				ctx,
			},
		))
	}
}
