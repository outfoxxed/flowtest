use proc_macro2::Ident;
use syn::{
	parenthesized,
	parse::{discouraged::Speculative, Parse, ParseStream},
	punctuated::Punctuated,
	Pat,
	PatIdent,
	Path,
	Token,
};

pub struct AttributeOptions {
	pub executor: Option<Path>,
	pub inputs: Punctuated<Input, Token![,]>,
	pub result_override: Option<bool>,
}

struct InnerOptions {
	pub executor: Option<Path>,
	pub inputs: Punctuated<Input, Token![,]>,
}

pub struct Input {
	pub from: Ident,
	pub pat: Pat,
}

impl Parse for InnerOptions {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let executor = 'executor: {
			let einput = input.fork();
			let Ok(name) = einput.parse::<Ident>() else { break 'executor None };

			if name != "executor" {
				break 'executor None;
			}

			if einput.parse::<Token![=]>().is_err() {
				break 'executor None;
			}

			let executor = einput.parse::<Path>()?;

			if einput.is_empty() {
				return Ok(Self {
					executor: Some(executor),
					inputs: Punctuated::new(),
				})
			}

			let _ = einput.parse::<Token![,]>()?;
			input.advance_to(&einput);
			Some(executor)
		};

		let inputs = Punctuated::<Input, Token![,]>::parse_terminated(input)?;
		Ok(Self { executor, inputs })
	}
}

impl Parse for AttributeOptions {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let with_parens = (|| {
			let content;
			parenthesized!(content in input);
			content.parse::<InnerOptions>()
		})();

		if let Ok(InnerOptions { executor, inputs }) = with_parens {
			let parse_resultopt = |input: ParseStream| {
				if input.parse::<Token![->]>().is_err() {
					return Ok(None);
				}

				let ret = input.parse::<Ident>()?;

				match &ret.to_string() as &str {
					"result" => Ok(Some(true)),
					"data" => Ok(Some(false)),
					_ => Err(syn::Error::new_spanned(ret, "expected `result` or `data`")),
				}
			};

			Ok(Self {
				executor,
				inputs,
				result_override: parse_resultopt(input)?,
			})
		} else {
			let InnerOptions { executor, inputs } = input.parse()?;

			Ok(Self {
				executor,
				inputs,
				result_override: None,
			})
		}
	}
}

impl Parse for Input {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let from = input.parse::<Ident>()?;
		let pat = match input.parse::<Token![:]>() {
			Ok(_) => Pat::parse_single(input)?,
			Err(_) => Pat::Ident(PatIdent {
				attrs: Vec::new(),
				by_ref: None,
				mutability: None,
				ident: from.clone(),
				subpat: None,
			}),
		};

		Ok(Input { from, pat })
	}
}
