//! Proc-macro crate for [`dx-preview`](https://docs.rs/dx-preview).
//!
//! Provides two macros:
//!
//! - [`preview`] — attribute macro that registers a Dioxus component in the
//!   `dx-preview` compile-time inventory.
//! - [`Reflect`] — derive macro that implements
//!   [`Reflect`](dx_preview::model::Reflect) for unit enum types, making them
//!   usable as editable properties in the preview shell.
//!
//! Provides the [`preview`] attribute macro, which registers a Dioxus component
//! in the `dx-preview` compile-time inventory and generates the plumbing needed
//! by the interactive preview shell.
//!
//! # What the macro generates
//!
//! For each annotated component the macro emits three items alongside the
//! original function (all gated on `#[cfg(feature = "preview")]`):
//!
//! The `crate` argument can be used when the macro is applied inside the
//! `dx-preview` crate itself, to avoid the circular `::dx_preview` path:
//!
//! ```rust,ignore
//! #[dx_preview::preview(crate = crate)]
//! #[component]
//! pub fn MyInput(…) -> Element { … }
//! ```
//!
//! 1. A `static [Property; N]` array — one entry per *visible* parameter,
//!    describing its name, reflected [`Type`](dx_preview::model::Type), and
//!    whether it is required.
//! 2. A `fn() -> Vec<Value>` — returns the default value for each visible
//!    parameter.
//! 3. A `fn(Vec<Value>) -> Element` — reconstructs the component props
//!    from the supplied values and calls the component.
//!
//! These are registered via `inventory::submit!` so that the shell can discover
//! every annotated component without a central list.

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
	Data, DeriveInput, Fields, FnArg, ItemFn, Pat, Token, Variant, parse::ParseStream,
	parse_macro_input,
};

// ── Macro attribute args ──────────────────────────────────────────────────────

struct PreviewArgs {
	/// Override for the `dx_preview` crate path, e.g. `crate` when the macro
	/// is used inside `dx-preview` itself.
	krate: Option<syn::Path>,
}

impl syn::parse::Parse for PreviewArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.is_empty() {
			return Ok(Self { krate: None });
		}
		// Accept `crate = <path>`.  The key is the `crate` keyword.
		let _: Token![crate] = input.parse()?;
		let _: Token![=] = input.parse()?;
		let path: syn::Path = input.parse()?;
		Ok(Self { krate: Some(path) })
	}
}

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

// ── Reflect derive ────────────────────────────────────────────────────────────

/// Derives [`Reflect`](dx_preview::model::Reflect) for a **unit enum**.
///
/// Each variant is mapped to a `u8` index (in declaration order). The
/// resulting [`Type`](dx_preview::model::Type) is
/// [`Type::Enum`](dx_preview::model::Type::Enum), which the preview shell
/// renders as a `<select>` dropdown.
///
/// # Panics
///
/// Compilation fails if:
/// - The type is not an enum.
/// - Any variant carries data fields.
/// - The enum has more than 256 variants.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(dx_preview::Reflect)]
/// enum Color { Red, Green, Blue }
/// ```
#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let name = &input.ident;
	let name_str = name.to_string();

	let Data::Enum(data) = &input.data else {
		return syn::Error::new_spanned(&input, "`Reflect` can only be derived for enums")
			.to_compile_error()
			.into();
	};

	let variants: Vec<&Variant> = data.variants.iter().collect();

	for variant in &variants {
		if !matches!(variant.fields, Fields::Unit) {
			return syn::Error::new_spanned(
				variant,
				"`Reflect` only supports unit enum variants (no fields)",
			)
			.to_compile_error()
			.into();
		}
	}

	if variants.len() > 256 {
		return syn::Error::new_spanned(&input, "`Reflect` supports at most 256 enum variants")
			.to_compile_error()
			.into();
	}

	let variant_names: Vec<String> = variants.iter().map(|v| v.ident.to_string()).collect();
	let variant_idents: Vec<&Ident> = variants.iter().map(|v| &v.ident).collect();
	let variant_indices: Vec<u8> = (0u8..variants.len() as u8).collect();

	quote! {
		impl ::dx_preview::model::Reflect for #name {
			const TYPE: ::dx_preview::model::Type = ::dx_preview::model::Type::Enum(
				::dx_preview::model::EnumType {
					name: #name_str,
					variants: &[
						#( ::dx_preview::model::EnumVariant { name: #variant_names } ),*
					],
				},
			);

			fn to_value(self) -> ::dx_preview::model::Value {
				::dx_preview::model::Value::Enum(match self {
					#( Self::#variant_idents => #variant_indices, )*
				})
			}

			fn try_from_value(
				value: ::dx_preview::model::Value,
			) -> Result<Self, ::dx_preview::model::TypeError> {
				match value {
					#( ::dx_preview::model::Value::Enum(#variant_indices) => Ok(Self::#variant_idents), )*
					_ => Err(::dx_preview::model::TypeError::InvalidValue),
				}
			}
		}
	}
	.into()
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
/// | *(none)* | Visible; type must implement `Reflect`; default via `Default::default()` |
/// | `#[preview(default = expr)]` | Visible; use `expr` as default |
/// | `#[preview(hide)]` | Hidden from UI; rendered with `Default::default()` |
/// | `#[preview(hide, default = expr)]` | Hidden from UI; rendered with `expr` |
///
/// # Crate path
///
/// When the macro is applied inside `dx-preview` itself, pass `crate = crate`
/// so the generated code resolves to the local crate rather than an external
/// `::dx_preview` path:
///
/// ```rust,ignore
/// #[dx_preview::preview(crate = crate)]
/// #[component]
/// pub fn MyInput(…) -> Element { … }
/// ```
///
/// # Example
/// ```rust,ignore
/// #[dx_preview::preview]
/// #[component]
/// pub fn MyButton(
///     #[preview(default = "Click me".to_string())] label: String,
///     #[preview(hide)] onclick: Option<Callback<MouseEvent>>,
///     #[preview(hide, default = rsx! { span { "→" } })] children: Element,
/// ) -> Element { ... }
/// ```
#[proc_macro_attribute]
pub fn preview(args: TokenStream, input: TokenStream) -> TokenStream {
	let preview_args = parse_macro_input!(args as PreviewArgs);
	let krate = preview_args
		.krate
		.map(|p| quote! { #p })
		.unwrap_or_else(|| quote! { ::dx_preview });
	// Convenience alias for the model module path used throughout.
	let model = quote! { #krate::model };

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
	let render_fn = Ident::new(
		&format!("__dx_preview_render_{}", fn_name_str),
		Span::call_site(),
	);

	// ── Visible params ────────────────────────────────────────────────────────

	let visible: Vec<&ParamInfo> = params.iter().filter(|p| !p.hidden).collect();
	let visible_count = visible.len();

	// Per-property default functions and Property static entries.
	let mut default_fns: Vec<TokenStream2> = Vec::new();
	let prop_entries: Vec<TokenStream2> = visible
		.iter()
		.enumerate()
		.map(|(i, p)| {
			let name_str = p.name.to_string();
			let ty = &p.ty;
			let default = p.default_tokens();

			// Generate a named fn so it can be stored as a fn() -> Value.
			let default_fn_name = Ident::new(
				&format!("__dx_preview_default_{}_{}", fn_name_str, i),
				Span::call_site(),
			);

			default_fns.push(quote! {
				#[cfg(feature = "preview")]
				#[doc(hidden)]
				#[allow(non_snake_case)]
				fn #default_fn_name() -> #model::Value {
					<#ty as #model::Reflect>::to_value(#default)
				}
			});

			quote! {
				#model::Property {
					name: #name_str,
					r#type: <#ty as #model::Reflect>::TYPE,
					required: true,
					default_value: #default_fn_name,
				}
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
						<#ty as #model::Reflect>::try_from_value(
							__values.next().unwrap_or(#model::Value::Option(None)),
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

		// Everything below is only compiled when the `preview` feature is
		// active, which enables `dx-preview/app` and the book types.
		#(#default_fns)*

		#[cfg(feature = "preview")]
		#[doc(hidden)]
		#[allow(non_upper_case_globals)]
		static #props_static: [#model::Property; #visible_count] = [
			#(#prop_entries),*
		];

		#[cfg(feature = "preview")]
		#[doc(hidden)]
		#[allow(non_snake_case)]
		fn #render_fn(
			values: ::std::vec::Vec<#model::Value>,
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

		#[cfg(feature = "preview")]
		#model::inventory::submit! {
			#model::ComponentEntry {
				name: #fn_name_str,
				properties: &#props_static,
				render: #render_fn,
			}
		}
	}
	.into()
}
