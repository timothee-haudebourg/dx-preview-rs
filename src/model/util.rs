use dioxus::logger::tracing::trace;
use dioxus::{
	core::{ReactiveContext, Runtime, ScopeId},
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
	B: Writable + Scoped + Copy + 'static,
	A::Target: 'static + PartialEq + Sized,
	B::Target: 'static + PartialEq + Sized,
{
	let scope = highest_scope(a.scope(), b.scope());

	trace!("couple_signals: watching a→b");
	watch(a, scope, move || {
		if let Some(new_b) = a_to_b(&*a.peek())
			&& b.try_peek().is_ok_and(|cur| *cur != new_b)
		{
			let mut b = b;
			b.set(new_b);
		}
	});
	trace!("couple_signals: watching b→a");
	watch(b, scope, move || {
		if let Some(new_a) = b_to_a(&*b.peek())
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
fn watch<R>(signal: R, scope: ScopeId, mut callback: impl 'static + FnMut())
where
	R: Writable + Scoped,
	R::Target: 'static,
{
	let rt = Runtime::current();
	let (send, mut recv) = unbounded::<()>();

	trace!("watch: spawning task in scope {:?}", scope);
	rt.spawn(scope, async move {
		while recv.next().await.is_some() {
			callback();
		}
	});
	trace!("watch: creating ReactiveContext");
	let ctx = ReactiveContext::new_with_callback(
		move || {
			let _ = send.unbounded_send(());
		},
		scope,
		std::panic::Location::caller(),
	);
	trace!("watch: subscribing ReactiveContext to signal");
	ctx.subscribe(signal.subscribers());
	trace!("watch: done");
}
