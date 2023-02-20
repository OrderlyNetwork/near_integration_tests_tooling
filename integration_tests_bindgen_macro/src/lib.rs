extern crate proc_macro;
mod generate_test_bind;
mod parse;
mod types;
use generate_test_bind::{generate_impl, generate_struct};
use parse::{parse_func_info, parse_struct_info};
use proc_macro::{Span, TokenStream};
use syn::{Attribute, ItemImpl, ItemStruct};

macro_rules! compile_error {
    ($($tt:tt)*) => {
        return syn::Error::new(Span::call_site().into(), format!($($tt)*)).to_compile_error().into()
    }
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[proc_macro_attribute]
pub fn integration_tests_bindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(item) = syn::parse::<ItemStruct>(input.clone()) {
        if is_marked_near_bindgen(&item.attrs) {
            let struct_info = parse_struct_info(item);
            generate_struct(input.into(), struct_info).into()
        } else {
            compile_error!("integration_tests_bind_gen can only be used in pair with near_bindgen.")
        }
    } else if let Ok(item) = syn::parse::<ItemImpl>(input.clone()) {
        if is_marked_near_bindgen(&item.attrs) {
            let func_info = parse_func_info(item);
            generate_impl(input.into(), func_info).into()
        } else {
            compile_error!("integration_tests_bind_gen can only be used in pair with near_bindgen.")
        }
    } else {
        compile_error!(
            "integration_tests_bind_gen can only be used on type declarations and impl sections."
        )
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[proc_macro_attribute]
pub fn integration_tests_bindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

fn is_marked_near_bindgen(attrs: &[Attribute]) -> bool {
    has_attribute(attrs, "near_bindgen")
}

pub(crate) fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .map(|attr| attr.parse_meta())
        .any(|res| match res {
            Ok(meta) => meta
                .path()
                .get_ident()
                .map(|el| el.to_string())
                .filter(|el| el == name)
                .is_some(),
            Err(_) => false,
        })
}
