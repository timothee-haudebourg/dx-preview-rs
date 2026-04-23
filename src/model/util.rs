use dioxus::{
	core::{ReactiveContext, Runtime, ScopeId},
	logger::tracing::trace,
	signals::{ReadableExt, Signal, Writable, WritableExt},
};
use futures_channel::mpsc::unbounded;
use futures_util::StreamExt;

pub trait Scoped {
	fn scope(&self) -> ScopeId;
}

impl<T: 'static> Scoped for Signal<T> {
	fn scope(&self) -> ScopeId {
		self.origin_scope()
	}
}

/// Couple two readable/writable values bidirectionally without hooks.
///
/// Whenever `a` changes, `a_to_b` is called and — if it returns `Some` and
/// the value differs — the result is written to `b`. The symmetric watcher
/// does the same from `b` to `a`. Equality checks prevent feedback loops.
///
/// The watchers are owned by the scope with the shorter lifetime (deepest in
/// the tree), so they are cleaned up when either value's scope is dropped.
pub fn couple_signals<A, B>(
	a: A,
	b: B,
	a_to_b: impl Fn(&A::Target) -> Option<B::Target> + 'static,
	b_to_a: impl Fn(&B::Target) -> Option<A::Target> + 'static,
) where
	A: Writable + Scoped + Copy + 'static,
	B: Writable + Scoped + Clone + 'static,
	A::Target: 'static + PartialEq + Sized,
	B::Target: 'static + PartialEq + Sized,
{
	let task_scope = highest_scope(a.scope(), b.scope());
	// ctx_scope must be stable — use the ReactiveValue's context scope
	// (= ValueContext's scope) which outlives individual component scopes.
	let ctx_scope = b.scope();

	// a→b watcher: subscribes to `a`, writes to a clone of `b`
	let mut b_for_atob = b.clone();
	watch(a, task_scope, ctx_scope, move || {
		if let Some(new_b) = a_to_b(&*a.peek())
			&& b_for_atob.try_peek().is_ok_and(|cur| *cur != new_b)
		{
			b_for_atob.set(new_b);
		}
	});

	// b→a watcher: subscribes to `b` (moved), writes to `a`
	// The callback also needs to read `b`, so clone one for the callback.
	let b_for_btoa_cb = b.clone();
	watch(b, task_scope, ctx_scope, move || {
		if let Some(new_a) = b_to_a(&*b_for_btoa_cb.peek())
			&& a.try_peek().is_ok_and(|cur| *cur != new_a)
		{
			let mut a = a;
			a.set(new_a);
		}
	});
	trace!("couple_signals: done");
}

/// Returns the scope with the highest height (deepest in tree = shortest lifetime).
/// Watchers owned by this scope are cleaned up when either signal's scope is dropped.
fn highest_scope(a: ScopeId, b: ScopeId) -> ScopeId {
	let rt = Runtime::current();
	if rt.height(a) >= rt.height(b) { a } else { b }
}

/// Subscribe `callback` to `signal` changes, running it in `scope`.
///
/// A [`ReactiveContext`] is subscribed to `signal`'s subscriber list; whenever
/// the signal is written to, the context fires the callback via an async channel
/// task, avoiding the `Send + Sync` constraint that direct closure capture would
/// impose.
/// * `task_scope` — the scope that owns the async task (determines its lifetime).
/// * `ctx_scope` — the scope associated with the [`ReactiveContext`]; must
///   outlive the watcher. Use the [`ReactiveValue`]'s stable context scope so
///   that `rt.needs_update(ctx_scope)` never panics on a removed scope.
fn watch<R>(
	signal: R,
	task_scope: ScopeId,
	ctx_scope: ScopeId,
	mut callback: impl 'static + FnMut(),
) where
	R: Writable + Scoped + 'static,
	R::Target: 'static,
{
	let rt = Runtime::current();
	let (send, mut recv) = unbounded::<()>();

	let ctx = ReactiveContext::new_with_callback(
		move || {
			let _ = send.unbounded_send(());
		},
		ctx_scope,
		std::panic::Location::caller(),
	);

	trace!("watch: spawning task in scope {:?}", task_scope);

	// Defer the subscription until the task is first polled — after the
	// current render's with_scope borrow is released — to avoid a
	// "RefCell already borrowed" panic when couple_signals is called
	// during rendering.
	rt.spawn(task_scope, async move {
		trace!("watch: subscribing ReactiveContext to signal (deferred)");
		ctx.subscribe(signal.subscribers());
		trace!("watch: subscribed, entering loop");
		while recv.next().await.is_some() {
			callback();
		}
	});
	trace!("watch: done");
}
