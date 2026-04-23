use std::{
	cell::RefCell,
	collections::{HashMap, hash_map::Entry},
	rc::Rc,
};

use dioxus::{core::current_scope_id, prelude::*};
use slab::Slab;

use crate::model::Value;

use super::{ReactiveValue, ReactiveValueId, ValueSource};

/// Stores every [`ReactiveValue`] signal for one window.
///
/// Provide once per window via [`dioxus::prelude::provide_context`]; accessed
/// by [`ReactiveValue`] through [`try_consume_context`].
pub(crate) struct ValueContext {
	side: ValueSource,

	/// Scope in which this context was provided; signals are created here so
	/// their lifetime matches the context rather than any individual component.
	scope_id: ScopeId,

	/// Locally-owned entries. The [`Slab`] key is the signal id.
	local: RefCell<Slab<SignalEntry>>,

	/// Remotely-owned entries.
	remote: RefCell<HashMap<usize, SignalEntry>>,

	/// Called when this window writes a value locally (to propagate to remote).
	on_write: Box<dyn Fn(ReactiveValueId, &Value)>,
}

impl ValueContext {
	/// Create a new context. Must be called from a component/hook scope so
	/// [`current_scope_id`] can capture the right owner for new signals.
	pub fn new(side: ValueSource, on_write: impl 'static + Fn(ReactiveValueId, &Value)) -> Self {
		debug!("[{side}] init value context");

		Self {
			side,
			scope_id: current_scope_id(),
			local: RefCell::new(Slab::new()),
			remote: RefCell::new(HashMap::new()),
			on_write: Box::new(on_write),
		}
	}

	/// Create a new local signal.
	pub fn create(self: &Rc<Self>, initial: Value) -> ReactiveValue {
		let signal = self.new_signal(initial);
		let id = self.local.borrow_mut().insert(SignalEntry {
			signal,
			local_refs: 1,
		});

		debug!("[{}] create {id}", self.side);

		ReactiveValue {
			id: ReactiveValueId {
				owner: self.side,
				id,
			},
			signal,
			ctx: self.clone(),
		}
	}

	/// Apply a received value to a signal.
	///
	/// - **Remote-owned, unknown**: creates a new entry with `value` as its
	///   initial value.
	/// - **Remote-owned, known**: updates the stored value (no-op if equal).
	/// - **Locally-owned, known**: updates the stored value (no-op if equal).
	/// - **Locally-owned, unknown**: panics — local signals must be created via
	///   [`create`]; receiving a reference to a non-existent local signal
	///   indicates a bug.
	pub fn set(&self, id: ReactiveValueId, value: Value) -> Signal<Value> {
		debug!("[{}] set {id} = {value:?}", self.side);

		if id.owner == self.side {
			let mut signal = self
				.get_local_signal(id.id)
				.expect("locally-owned signal not found — it may have been freed");

			if *signal.peek() != value {
				signal.set(value);
			}

			signal
		} else {
			let mut remote = self.remote.borrow_mut();

			match remote.entry(id.id) {
				Entry::Vacant(e) => {
					let signal = self.new_signal(value);

					e.insert(SignalEntry {
						signal,
						local_refs: 1,
					});

					signal
				}
				Entry::Occupied(e) => {
					let mut signal = e.into_mut().signal;

					if *signal.peek() != value {
						signal.set(value);
					}

					signal
				}
			}
		}
	}

	/// Fire the `on_write` callback for `id` with the given `value`.
	///
	/// Used by [`ReactiveWriteRef`](super::ReactiveWriteRef) on drop to notify
	/// the remote window of a change made through a write reference.
	pub(super) fn notify_write(&self, id: ReactiveValueId, value: &Value) {
		(self.on_write)(id, value);
	}

	// ── Private helpers ───────────────────────────────────────────────────────

	/// Create a new [`Signal`] whose lifetime is tied to this context's scope.
	fn new_signal(&self, initial: Value) -> Signal<Value> {
		Signal::new_in_scope(initial, self.scope_id)
	}

	/// Look up the signal for `id` in either the local or remote map.
	pub fn get_signal(&self, id: ReactiveValueId) -> Option<Signal<Value>> {
		if id.owner == self.side {
			self.get_local_signal(id.id)
		} else {
			self.get_remote_signal(id.id)
		}
	}

	pub fn get_local_signal(&self, id: usize) -> Option<Signal<Value>> {
		self.local.borrow().get(id).map(|e| e.signal)
	}

	pub fn get_remote_signal(&self, id: usize) -> Option<Signal<Value>> {
		self.remote.borrow().get(&id).map(|e| e.signal)
	}

	pub fn inc_ref(&self, id: ReactiveValueId) {
		if id.owner == self.side {
			if let Some(e) = self.local.borrow_mut().get_mut(id.id) {
				e.local_refs += 1;
			}
		} else if let Some(e) = self.remote.borrow_mut().get_mut(&id.id) {
			e.local_refs += 1;
		}
	}

	pub fn dec_ref(&self, id: ReactiveValueId) {
		if id.owner == self.side {
			let remove = {
				let mut local = self.local.borrow_mut();
				match local.get_mut(id.id) {
					None => return,
					Some(e) => {
						e.local_refs = e.local_refs.saturating_sub(1);
						e.local_refs == 0
					}
				}
			};
			if remove {
				self.local.borrow_mut().remove(id.id);
			}
		} else {
			let remove = {
				let mut remote = self.remote.borrow_mut();
				match remote.get_mut(&id.id) {
					None => return,
					Some(e) => {
						e.local_refs = e.local_refs.saturating_sub(1);
						e.local_refs == 0
					}
				}
			};
			if remove {
				self.remote.borrow_mut().remove(&id.id);
			}
		}
	}
}

struct SignalEntry {
	signal: Signal<Value>,
	local_refs: usize,
}
