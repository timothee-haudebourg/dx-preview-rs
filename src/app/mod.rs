//! Interactive preview shell.
//!
//! Serves two routes:
//!
//! - **`/`** — the shell: a sidebar listing all registered components and a
//!   property editor panel. The selected component is displayed in an `<iframe>`.
//! - **`/component/:name?props=…`** — the isolation page: renders just the
//!   component with the given property values, with no shell chrome, so its
//!   styles are fully isolated from the rest of the UI.

use dioxus::prelude::*;

mod component;
mod config;
mod home;
pub mod protocol;
pub mod ui;

use component::ComponentPage;
pub use config::Config;
use home::Home;

/// Launch the storybook web app with the given configuration.
///
/// All components annotated with `#[dx_preview::preview]` in crates compiled with
/// the `storybook` feature are automatically discovered and shown in the sidebar.
pub fn launch(config: Config) {
	config.apply();
	dioxus::launch(App);
}

#[derive(Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[route("/")]
    Home {},

    #[route("/component/:name?:props")]
    ComponentPage { name: String, props: String },
}

#[component]
fn App() -> Element {
	rsx! {
		Router::<Route> {}
	}
}
