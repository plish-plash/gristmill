use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Ident, Lit, Meta, MetaNameValue};

fn find_category_attribute(attrs: Vec<Attribute>) -> syn::parse::Result<Ident> {
    for attr in attrs {
        if attr.path.is_ident("asset_category") {
            let meta = attr.parse_meta()?;
            if let Meta::NameValue(MetaNameValue {
                lit: Lit::Str(name),
                ..
            }) = meta
            {
                return name.parse();
            } else {
                panic!("invalid format for asset_category attribute");
            }
        }
    }
    Ok(Ident::new("ASSET", Span::call_site()))
}

#[proc_macro_derive(Asset, attributes(asset_category))]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let category = find_category_attribute(input.attrs).unwrap();
    let output = quote! {
        impl gristmill::asset::Asset for #name {
            fn category() -> gristmill::asset::AssetCategory {
                gristmill::asset::AssetCategory::#category
            }
            fn read_from(reader: gristmill::asset::BufReader) -> gristmill::asset::AssetResult<Self> {
                gristmill::asset::util::read_yaml(reader)
            }
        }
    };
    TokenStream::from(output)
}

#[proc_macro_derive(AssetWrite)]
pub fn derive_asset_write(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let output = quote! {
        impl gristmill::asset::AssetWrite for #name {
            fn write_to(value: &Self, writer: gristmill::asset::BufWriter) -> gristmill::asset::AssetResult<()> {
                gristmill::asset::util::write_yaml(writer, value)
            }
        }
    };
    TokenStream::from(output)
}
