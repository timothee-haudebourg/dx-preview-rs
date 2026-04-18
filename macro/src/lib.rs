//! Proc-macro crate for [`dx-preview`](https://docs.rs/dx-preview).
//!
//! Provides the [`preview`] attribute macro, which registers a Dioxus component
//! in the `dx-preview` compile-time inventory and generates the plumbing needed
//! by the interactive preview shell.
//!
//! # What the macro generates
//!
//! For each annotated component the macro emits three items alongside the
//! original function (all gated on `#[cfg(feature = "storybook")]`):
//!
//! 1. A `static [Property; N]` array — one entry per *visible* parameter,
//!    describing its name, reflected [`Type`](dx_preview::model::Type), and
//!    whether it is required.
//! 2. A `fn() -> Vec<Option<Value>>` — returns the default value (or `None` for
//!    optional properties that default to absent) for each visible parameter.
//! 3. A `fn(Vec<Option<Value>>) -> Element` — reconstructs the component props
//!    from the supplied values and calls the component.
//!
//! These are registered via `inventory::submit!` so that the shell can discover
//! every annotated component without a central list.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{FnArg, ItemFn, Pat, Token, parse::ParseStream, parse_macro_input};

// ── Attribute parsing ─────────────────────────────────────────────────────────

struct DemoAttrs {
	/// `#[demo(hide)]` — exclude from UI properties, only used in render.
	hidden: bool,
	/// `#[demo(default = <expr>)]` — explicit default value.
	default_expr: Option<TokenStream2>,
}

/// Strip all `#[preview(...)]` attributes from `attrs`, collecting their content.
/// Supports both `#[preview(hide)]` and `#[preview(default = expr)]`, and the
/// combined form `#[preview(hide, default = expr)]`.
fn take_preview_attrs(attrs: &mut Vec<syn::Attribute>) -> DemoAttrs {
	let mut hidden = false;
	let mut default_expr = None;

	attrs.retain(|attr| {
		if !attr.path().is_ident("preview") {
			return true; // keep non-preview attrs
		}
		let _ = attr.parse_args_with(|input: ParseStream| -> syn::Result<()> {
			while !input.is_empty() {
				let ident: syn::Ident = input.parse()?;
				if ident == "hide" {
					hidden = true;
				} else if ident == "default" {
					let _: Token![=] = input.parse()?;
					let expr: syn::Expr = input.parse()?;
					default_expr = Some(quote! { #expr });
				} else {
					return Err(syn::Error::new(
						ident.span(),
						"expected `hide` or `default = <expr>`",
					));
				}
				if input.peek(Token![,]) {
					let _: Token![,] = input.parse()?;
				}
			}
			Ok(())
		});
		false // remove all #[preview(...)] attrs
	});

	DemoAttrs {
		hidden,
		default_expr,
	}
}

// ── Parameter info ────────────────────────────────────────────────────────────

struct ParamInfo {
	name: Ident,
	ty: syn::Type,
	/// Exclude from `properties` / `default_values`; use default in render.
	hidden: bool,
	/// Explicit default expression (used both in UI defaults and render fallback).
	default_expr: Option<TokenStream2>,
}

impl ParamInfo {
	/// The token stream for this param's default value.
	fn default_tokens(&self) -> TokenStream2 {
		match &self.default_expr {
			Some(expr) => quote! { { #expr } },
			None => {
				let ty = &self.ty;
				quote! { <#ty as ::std::default::Default>::default() }
			}
		}
	}
}

// ── Proc macro ────────────────────────────────────────────────────────────────

/// Registers a Dioxus component in the dx-preview compile-time inventory.
///
/// Place this **above** `#[component]`. The macro strips `#[preview(...)]`
/// attributes from each parameter, then generates:
///
/// - A static `[Property; N]` describing the visible, reflectable parameters.
/// - A `fn() -> Vec<Value>` returning their default values.
/// - A `fn(Vec<Value>) -> Element` rendering the component from those values.
///
/// # Parameter attributes
///
/// | Attribute | Meaning |
/// |---|---|
/// | *(none)* | Visible; type must implement `ShowcaseType`; default via `Default::default()` |
/// | `#[preview(default = expr)]` | Visible; use `expr` as default |
/// | `#[preview(hide)]` | Hidden from UI; rendered with `Default::default()` |
/// | `#[preview(hide, default = expr)]` | Hidden from UI; rendered with `expr` |
///
/// # Example
/// ```rust
/// #[dx_preview::preview]
/// #[component]
/// pub fn MyButton(
///     #[preview(default = "Click me".to_string())] label: String,
///     #[preview(hide)] onclick: Option<Callback<MouseEvent>>,
///     #[preview(hide, default = rsx! { span { "→" } })] children: Element,
/// ) -> Element { ... }
/// ```
#[proc_macro_attribute]
pub fn preview(_args: TokenStream, input: TokenStream) -> TokenStream {
	let mut input_fn = parse_macro_input!(input as ItemFn);

	let fn_name = input_fn.sig.ident.clone();
	let fn_name_str = fn_name.to_string();
	// Call-site ident so the component name resolves in the call-site's module.
	let component_ident = Ident::new(&fn_name_str, Span::call_site());

	// ── Collect parameter info ────────────────────────────────────────────────

	let mut params: Vec<ParamInfo> = Vec::new();

	for arg in &mut input_fn.sig.inputs {
		if let FnArg::Typed(pat_type) = arg {
			let name = match &*pat_type.pat {
				Pat::Ident(p) => p.ident.clone(),
				_ => continue,
			};
			let ty = (*pat_type.ty).clone();
			let demo = take_preview_attrs(&mut pat_type.attrs);
			params.push(ParamInfo {
				name,
				ty,
				hidden: demo.hidden,
				default_expr: demo.default_expr,
			});
		}
	}

	// ── Derived identifiers ───────────────────────────────────────────────────

	let props_static = Ident::new(
		&format!("__dx_preview_props_{}", fn_name_str),
		Span::call_site(),
	);
	let defaults_fn = Ident::new(
		&format!("__dx_preview_defaults_{}", fn_name_str),
		Span::call_site(),
	);
	let render_fn = Ident::new(
		&format!("__dx_preview_render_{}", fn_name_str),
		Span::call_site(),
	);

	// ── Visible params ────────────────────────────────────────────────────────

	let visible: Vec<&ParamInfo> = params.iter().filter(|p| !p.hidden).collect();
	let visible_count = visible.len();

	// static [Property; N] entries
	let prop_entries: Vec<TokenStream2> = visible
		.iter()
		.map(|p| {
			let name_str = p.name.to_string();
			let ty = &p.ty;
			quote! {
				::dx_preview::model::Property {
					name: #name_str,
					r#type: <<#ty as ::dx_preview::model::PropertyType>::Type
						as ::dx_preview::model::Reflect>::TYPE,
					required: <#ty as ::dx_preview::model::PropertyType>::REQUIRED,
				}
			}
		})
		.collect();

	// default_values fn body: one entry per visible param
	let default_value_entries: Vec<TokenStream2> = visible
		.iter()
		.map(|p| {
			let ty = &p.ty;
			let default = p.default_tokens();
			quote! {
				<#ty as ::dx_preview::model::PropertyType>::to_opt_value(&#default)
			}
		})
		.collect();

	// ── Render body ───────────────────────────────────────────────────────────

	// For each param: extract from the values iterator (visible) or use default (hidden).
	let prop_extractions: Vec<TokenStream2> = params
		.iter()
		.map(|p| {
			let name = &p.name;
			let ty = &p.ty;
			let fallback = p.default_tokens();

			if p.hidden {
				quote! { let #name: #ty = #fallback; }
			} else {
				quote! {
					let #name: #ty =
						<#ty as ::dx_preview::model::PropertyType>::try_from_opt_value(
							__values.next().flatten(),
						)
						.unwrap_or_else(|_| #fallback);
				}
			}
		})
		.collect();

	// RSX prop assignments (explicit `name: name` form for clarity)
	let prop_assignments: Vec<TokenStream2> = params
		.iter()
		.map(|p| {
			let name = &p.name;
			quote! { #name: #name, }
		})
		.collect();

	// ── Output ────────────────────────────────────────────────────────────────

	quote! {
		// The original component, with #[preview(...)] stripped so that
		// #[component] sees a clean signature.
		#input_fn

		// Everything below is only compiled when the `storybook` feature is
		// active, which enables `dx-preview/app` and the book types.
		#[cfg(feature = "storybook")]
		#[doc(hidden)]
		#[allow(non_upper_case_globals)]
		static #props_static: [::dx_preview::model::Property; #visible_count] = [
			#(#prop_entries),*
		];

		#[cfg(feature = "storybook")]
		#[doc(hidden)]
		#[allow(non_snake_case)]
		fn #defaults_fn() -> ::std::vec::Vec<::std::option::Option<::dx_preview::model::Value>> {
			vec![#(#default_value_entries),*]
		}

		#[cfg(feature = "storybook")]
		#[doc(hidden)]
		#[allow(non_snake_case)]
		fn #render_fn(
			values: ::std::vec::Vec<::std::option::Option<::dx_preview::model::Value>>,
		) -> ::dioxus::prelude::Element {
			use ::dioxus::prelude::*;
			let mut __values = values.into_iter();
			#(#prop_extractions)*
			rsx! {
				#component_ident {
					#(#prop_assignments)*
				}
			}
		}

		#[cfg(feature = "storybook")]
		::dx_preview::model::inventory::submit! {
			::dx_preview::model::ComponentEntry {
				name: #fn_name_str,
				properties: &#props_static,
				default_values: #defaults_fn,
				render: #render_fn,
			}
		}
	}
	.into()
}
