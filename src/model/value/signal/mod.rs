use core::fmt;
use std::{
	hash::{Hash, Hasher},
	rc::Rc,
};

use dioxus::{
	core::{ScopeId, Subscribers, consume_context, try_consume_context, use_hook_with_cleanup},
	hooks::{use_context, use_context_provider},
	prelude::{ReadableRef, WritableRef, WriteLock},
	signals::{
		BorrowError, BorrowMutError, Readable, ReadableExt, Signal, UnsyncStorage, Writable,
	},
};

use crate::model::{Reflect, Scoped, TypeError, couple_signals};

use super::{Value, ValueSource};

mod context;

pub(crate) use context::ValueContext;

/// Provide a [`ValueContext`] for `side` and register `on_write` as the
/// callback to invoke whenever a [`ReactiveValue`] is written to locally.
///
/// The protocol passes a closure here that forwards updates to the remote
/// window. Must be called from a component/hook scope.
pub fn use_signal_value_context_provider(
	side: ValueSource,
	on_write: impl 'static + Fn(ReactiveValueId, &Value),
) {
	use_context_provider(move || Rc::new(ValueContext::new(side, on_write)));
}

/// Uniquely identifies a [`ReactiveValue`] across both windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ReactiveValueId {
	/// Which window created (owns) this signal.
	pub owner: ValueSource,
	/// Unique index within the owning window's [`Slab`] allocation.
	pub id: usize,
}

impl fmt::Display for ReactiveValueId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.id, self.owner)
	}
}

/// A reactive property value whose underlying signal is shared between windows.
///
/// The wire format is `{ id, value }`: the `value` snapshot lets the receiving
/// window register the signal on first sight. The `signal` field is stored
/// directly so reads never require a context lookup (avoiding panics when
/// Dioxus polls async tasks without a scope on the stack).
///
/// Equality and hashing compare only `id`.
#[derive(Clone)]
pub struct ReactiveValue {
	id: ReactiveValueId,

	/// The backing signal, stored directly so reads never need `consume_context`.
	signal: Signal<Value>,

	/// The context this value belongs to, stored directly so writes never need
	/// a context lookup (avoiding panics when Dioxus polls async tasks without
	/// a scope on the stack).
	ctx: Rc<ValueContext>,
}

impl ReactiveValue {
	/// Create a new reactive value backed by `initial`.
	///
	/// The signal is allocated in the [`ValueContext`]'s scope and lives until
	/// that scope drops. For lifecycle management tied to a specific component,
	/// use [`use_reactive_value`] instead.
	pub fn new(initial: Value) -> Self {
		let ctx = consume_context::<Rc<ValueContext>>();
		ctx.create(initial)
	}

	/// Apply a value without notifying the remote window.
	///
	/// Delegates to [`ValueContext::apply`]: creates the signal if it is a new
	/// remote signal, or updates the stored value otherwise. Used when
	/// receiving a [`WriteSignal`](crate::app::protocol) message to avoid
	/// echoing the notification back to the sender.
	pub fn write_silent(&self, value: Value) {
		self.ctx.set(self.id, value);
	}
}

pub trait AnyValueSignal: 'static + Clone + Scoped + Writable<Target = Value> {
	fn new(initial: Value) -> Self;

	fn into_couple_with<T>(self, signal: Signal<T>)
	where
		T: 'static + Clone + PartialEq + Reflect,
	{
		couple_signals(
			signal,
			self,
			|t: &T| Some(t.clone().to_value()),
			|v: &Value| T::try_from_value(v.clone()).ok(),
		);
	}

	fn couple_into_signal<T>(self) -> Result<Signal<T>, TypeError>
	where
		T: 'static + Clone + PartialEq + Reflect,
	{
		let initial = T::try_from_value(self.peek().clone())?;
		let result = Signal::new(initial);
		self.into_couple_with(result);
		Ok(result)
	}

	fn couple_from_signal<T>(signal: Signal<T>) -> Self
	where
		T: 'static + Clone + PartialEq + Reflect,
	{
		let initial = signal.peek().clone().to_value();
		let result = Self::new(initial);
		result.clone().into_couple_with(signal);
		result
	}
}

impl AnyValueSignal for ReactiveValue {
	fn new(initial: Value) -> Self {
		Self::new(initial)
	}
}

impl AnyValueSignal for Signal<Value> {
	fn new(initial: Value) -> Self {
		Self::new(initial)
	}
}

impl std::fmt::Debug for ReactiveValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReactiveValue")
			.field("id", &self.id)
			.finish_non_exhaustive()
	}
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

// Custom Serialize: bundle the current value by peeking directly through the
// stored signal handle — no `consume_context` required.
impl serde::Serialize for ReactiveValue {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		#[derive(serde::Serialize)]
		struct Wire<'a> {
			id: ReactiveValueId,
			value: &'a Value,
		}
		let value = (*self.signal.peek_unchecked()).clone();
		Wire {
			id: self.id,
			value: &value,
		}
		.serialize(serializer)
	}
}

// Custom Deserialize: extract the bundled value, register the signal if needed,
// and return a handle with the signal embedded.
impl<'de> serde::Deserialize<'de> for ReactiveValue {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		#[derive(serde::Deserialize)]
		struct Wire {
			id: ReactiveValueId,
			value: Value,
		}
		let Wire { id, value } = Wire::deserialize(deserializer)?;
		let ctx = try_consume_context::<Rc<ValueContext>>().ok_or_else(|| {
			serde::de::Error::custom("ReactiveValue cannot be deserialized outside a ValueContext")
		})?;
		ctx.set(id, value);
		let signal = ctx.get_signal(id).ok_or_else(|| {
			serde::de::Error::custom("ReactiveValue: signal not found after apply")
		})?;
		Ok(ReactiveValue { id, signal, ctx })
	}
}

/// Write metadata for [`ReactiveValue`].
pub struct ReactiveWriteMetadata {
	#[allow(dead_code)]
	inner_meta: <Signal<Value> as Writable>::WriteMetadata,

	value: ReactiveValue,
}

impl Drop for ReactiveWriteMetadata {
	fn drop(&mut self) {
		let value = self.value.signal.peek();
		self.value.ctx.notify_write(self.value.id, &*value);
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

impl Scoped for ReactiveValue {
	fn scope(&self) -> ScopeId {
		// The signal is created in the ValueContext's scope; its origin scope
		// is therefore the context's own scope.
		self.signal.origin_scope()
	}
}

impl Readable for ReactiveValue {
	type Target = Value;
	type Storage = UnsyncStorage;

	fn try_read_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
		self.signal.try_read_unchecked()
	}

	fn try_peek_unchecked(&self) -> Result<ReadableRef<'static, Self>, BorrowError> {
		self.signal.try_peek_unchecked()
	}

	fn subscribers(&self) -> Subscribers {
		self.signal.subscribers()
	}
}

impl Writable for ReactiveValue {
	type WriteMetadata = ReactiveWriteMetadata;

	fn try_write_unchecked(&self) -> Result<WritableRef<'static, Self>, BorrowMutError>
	where
		Self::Target: 'static,
	{
		let write = self.signal.try_write_unchecked()?;
		let (inner_write, inner_meta) = write.into_parts();

		Ok(WriteLock::new_with_metadata(
			inner_write,
			ReactiveWriteMetadata {
				inner_meta,
				value: self.clone(),
			},
		))
	}
}
