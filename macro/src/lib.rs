use std::mem;

use attribute::Input;
use quote::{format_ident, quote, ToTokens};
use syn::{
	parse_macro_input,
	punctuated::Punctuated,
	GenericArgument,
	ItemFn,
	PathArguments,
	ReturnType,
	Type,
	TypePath,
	TypeTuple,
};

use crate::attribute::AttributeOptions;

mod attribute;

#[proc_macro_attribute]
pub fn flowtest(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let mut attr = parse_macro_input!(attr as AttributeOptions);

	let ItemFn {
		attrs,
		vis,
		mut sig,
		block,
	} = parse_macro_input!(item as ItemFn);

	if let Some(asyncness) = sig.asyncness {
		return syn::Error::new_spanned(asyncness, "`async` is not allowed on flowtest tests. If you are using an async test attribute make sure it is above the flowtest attribute.")
			.into_compile_error()
			.into();
	}

	let executor = attr
		.executor
		.take()
		.map(ToTokens::into_token_stream)
		.unwrap_or_else(|| {
			quote! {
				::flowtest::standard_executor::StandardExecutor
			}
		});

	let continuation = format_ident!("__flowtest_test_{}", sig.ident);

	let mut out_ty = Type::Tuple(TypeTuple {
		paren_token: Default::default(),
		elems: Punctuated::new(),
	});

	let mut is_result = false;

	if let ReturnType::Type(_, ty) = &mut sig.output {
		'ret: {
			if attr.result_override != Some(false) {
				if let Type::Path(TypePath { qself: _, path }) = &mut **ty {
					if let Some(ty) = path.segments.last_mut() {
						// hopefully this should catch result types used in std and libraries like anyhow
						if attr.result_override == Some(true) || ty.ident.to_string() == "Result" {
							if let PathArguments::AngleBracketed(args) = &mut ty.arguments {
								if let Some(GenericArgument::Type(ty)) = args.args.first_mut() {
									mem::swap(ty, &mut out_ty);
									is_result = true;
									break 'ret
								}
							}
						}
					}
				}
			}

			mem::swap(&mut **ty, &mut out_ty);
		}

		if attr.result_override == Some(true) && !is_result {
			return syn::Error::new_spanned(ty, "unable to parse return type as result (required due to `-> result` in flowtest attribute)")
				.into_compile_error()
				.into()
		}
	} else if attr.result_override == Some(true) {
		return syn::Error::new_spanned(
			sig.ident,
			"function must return a result (required due to `-> result` in flowtest attribute)",
		)
		.into_compile_error()
		.into()
	}

	let exec_fn = match is_result {
		true => format_ident!("exec_result"),
		false => format_ident!("exec_noresult"),
	};

	let dependencies = attr.inputs.iter().map(|input| {
		let Input { from, pat } = input;

		let from_continuation = format_ident!("__flowtest_test_{}", from);

		quote! {
			let #pat = match ::flowtest::Executor::wait(&mut __flowtest_executor, &#from_continuation) {
				Ok(v) => v,
				Err(::flowtest::__private::TestFailedError) => ::std::panic!(
					concat!("flowtest dependency `", stringify!(#from), "` failed")
				),
			};
		}
	});

	let new_fn = quote! {
		#[allow(non_upper_case_globals)]
		static #continuation: <#executor as ::flowtest::Executor>::Continuation<::std::result::Result<#out_ty, ::flowtest::__private::TestFailedError>> =
			<<#executor as ::flowtest::Executor>::Continuation<::std::result::Result<#out_ty, ::flowtest::__private::TestFailedError>>
				as ::flowtest::Continuation>::INITIAL;

		#(#attrs)* #vis #sig {
			let mut __flowtest_executor = <#executor as ::flowtest::Executor>::init();

			#(#dependencies)*

			::flowtest::__private::#exec_fn(__flowtest_executor, &#continuation, move || #block)
		}
	};

	new_fn.into()
}
