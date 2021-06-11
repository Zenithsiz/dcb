//! Derive macros for `Bytes`
// TODO: Check why we can't link to `Bytes` here?

// Imports
use quote::ToTokens;

#[proc_macro_derive(ProxySentinel, attributes(proxy_sentinel))]
pub fn proxy_sentinel_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	//let args = syn::parse_macro_input!(args as syn::AttributeArgs);
	let input = syn::parse_macro_input!(input as syn::ItemStruct);

	// Get the field and report error if there's more than one or none
	let mut field_iter = input.fields.iter();
	let field = match (field_iter.next(), field_iter.next()) {
		(Some(field), None) => field,
		_ => {
			panic!("Struct must contain exactly 1 field");
		},
	};

	if field.ident.is_some() {
		panic!("Struct must be a tuple struct");
	}

	let mut sentinel_value = None;
	let mut wrapper_type = None;
	for attr in &input.attrs {
		match attr.parse_meta() {
			Ok(syn::Meta::List(list))
				if list
					.path
					.get_ident()
					.map(|ident| ident == "proxy_sentinel")
					.unwrap_or(false) =>
			{
				for nested_attr in &list.nested {
					match nested_attr {
						syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => match name_value.path.get_ident() {
							Some(ident) if ident == "value" => sentinel_value = Some(name_value.lit.clone()),
							Some(ident) if ident == "wrapper_type" => wrapper_type = Some(name_value.lit.clone()),
							Some(ident) => panic!("Unknown setting '{}'", ident),
							None => panic!("`proxy_sentinel` settings must be identifiers"),
						},
						_ => panic!("You must supply a value in `proxy_sentinel`"),
					}
				}
			},
			_ => continue,
		}
	}

	let sentinel_value = sentinel_value.expect("You must supply a sentinel value via `proxy_sentinel(value = ...)`");
	let wrapper_type = wrapper_type.expect("You must supply the wrapper type via `proxy_sentinel(wrapper_type = ...)`");
	// TODO: Do this better, it's awful
	let wrapper_type: syn::TypePath =
		syn::parse_str(wrapper_type.to_token_stream().to_string().trim_matches('"')).expect("");
	//let wrapper_type = syn::parse_macro_input!(wrapper_type_token_stream as );

	let struct_name = input.ident;
	let output = quote::quote!(
		#[allow(clippy::diverging_sub_expression)] // Errors might be `!`
		impl ::dcb_bytes::Bytes for #struct_name {
			type ByteArray = <#wrapper_type as ::dcb_bytes::Bytes>::ByteArray;

			type DeserializeError = <#wrapper_type as ::dcb_bytes::Bytes>::DeserializeError;
			fn deserialize_bytes(bytes: &Self::ByteArray) -> Result<Self, Self::DeserializeError>
			{
				match bytes {
					#sentinel_value => Ok( Self(None) ),
					_               => Ok( Self(Some( ::dcb_bytes::Bytes::deserialize_bytes(bytes)? )) ),
				}
			}

			type SerializeError = <#wrapper_type as ::dcb_bytes::Bytes>::SerializeError;
			fn serialize_bytes(&self, bytes: &mut Self::ByteArray) -> Result<(), Self::SerializeError>
			{
				match &self.0 {
					Some(value) => ::dcb_bytes::Bytes::serialize_bytes(value, bytes)?,
					None        => *bytes = #sentinel_value,
				}

				Ok(())
			}
		}
	);

	output.into()
}

#[proc_macro_derive(Discriminant)]
pub fn discriminant_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(input as syn::ItemEnum);

	let mut keys: Vec<&syn::Expr> = vec![];
	let mut values: Vec<&syn::Ident> = vec![];
	for variant in &input.variants {
		values.push(&variant.ident);

		match &variant.discriminant {
			Some((_, expr)) => keys.push(expr),
			None => panic!("All variants must have discriminants"),
		}
	}

	let struct_name = input.ident;
	let output = quote::quote!(
		// TODO: Maybe just define this in `dcb_bytes` and reference it here instead?
		/// Error type for [`Bytes::deserialize_bytes`]
		#[derive(PartialEq, Eq, Clone, Copy, Debug, ::thiserror::Error)]
		pub enum DeserializeBytesError {
			/// Unknown value
			#[error("Unknown byte {:#x}", byte)]
			UnknownValue {
				/// Unknown byte
				byte: u8,
			},
		}

		#[allow(clippy::diverging_sub_expression)] // Errors might be `!`
		impl ::dcb_bytes::Bytes for #struct_name {
			type ByteArray = u8;

			type DeserializeError = DeserializeBytesError;
			fn deserialize_bytes(byte: &Self::ByteArray) -> Result<Self, Self::DeserializeError> {
				match *byte {
					#(
						byte if byte == #keys => Ok(Self::#values),
					)*

					byte => Err(DeserializeBytesError::UnknownValue { byte }),
				}
			}

			type SerializeError = !;
			fn serialize_bytes(&self, byte: &mut Self::ByteArray) -> Result<(), Self::SerializeError> {
				*byte = match &self {
					#(
						Self::#values => #keys,
					)*
				};

				Ok(())
			}
		}
	);

	output.into()
}
