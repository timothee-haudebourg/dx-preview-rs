extern crate dx_preview;

fn main() {
	dioxus::logger::init(dioxus::logger::tracing::Level::DEBUG).expect("failed to init logger");
	dx_preview::launch(dx_preview::Config::default());
}
