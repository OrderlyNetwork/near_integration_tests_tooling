use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo};

pub(crate) fn generate_struct(input: TokenStream, struct_info: StructInfo) -> TokenStream {
    let name = format_ident!("{}Test", struct_info.struct_name);

    let mut generated_struct: TokenStream = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        pub struct #name {
    }}
    .into();

    generated_struct.extend(input.into_iter());
    generated_struct
}

pub(crate) fn generate_impl(input: TokenStream, impl_info: ImplInfo) -> TokenStream {
    let impl_name = impl_info.impl_name;
    let mut func_stream_vec = vec![];
    for func_info in impl_info.func_infos {
        match &func_info.mutability {
            Mutability::Mutable(payable) => match payable {
                Payable::Payable => func_stream_vec.push(generate_payable_function(func_info)),
                Payable::NonPayable => func_stream_vec.push(generate_call_function(func_info)),
            },
            Mutability::Immutable => {
                func_stream_vec.push(generate_view_function(func_info));
            }
        }
    }
    let mut output = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        impl #impl_name {
            #(#func_stream_vec)*
        }
    };
    output.extend(input);

    output
}

fn json_serialize(func_info: &FunctionInfo) -> TokenStream {
    let args: TokenStream = func_info
        .params_ident
        .iter()
        .fold(None, |acc: Option<TokenStream>, value| {
            let ident = value;
            let ident_str = ident.to_string();
            Some(match acc {
                None => quote! { #ident_str: #ident }.into(),
                Some(a) => quote! { #a, #ident_str: #ident }.into(),
            })
        })
        .unwrap_or_else(|| TokenStream::default()); // handle the case when their is no args

    quote! {
      let args = near_sdk::serde_json::json!({#args}).to_string().into_bytes();
    }
    .into()
}

pub(crate) fn generate_view_function(func_info: FunctionInfo) -> TokenStream {
    let serialize_args = json_serialize(&func_info);
    let name = func_info.function_name.clone();
    let name_str = func_info.function_name.to_string();
    let params = func_info.params.clone();
    let return_ident = quote! { -> integration_tests_toolset::ImmutablePendingTx };

    let func = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        pub fn #name(&self, #params) #return_ident {
            #serialize_args
            integration_tests_toolset::ImmutablePendingTx::new( String::from(#name_str), args)
        }
    };

    func
}

pub(crate) fn generate_call_function(func_info: FunctionInfo) -> TokenStream {
    let serialize_args = json_serialize(&func_info);
    let name = func_info.function_name.clone();
    let name_str = func_info.function_name.to_string();
    let params = func_info.params.clone();
    let return_ident = quote! { -> integration_tests_toolset::MutablePendingTx };

    let func = quote! {
        pub fn #name(&self, #params) #return_ident {
            #serialize_args
            integration_tests_toolset::MutablePendingTx::new( String::from(#name_str), args)
        }
    };

    func
}

pub(crate) fn generate_payable_function(func_info: FunctionInfo) -> TokenStream {
    let serialize_args = json_serialize(&func_info);
    let name = func_info.function_name.clone();
    let name_str = func_info.function_name.to_string();
    let params = func_info.params.clone();
    let return_ident = quote! { -> integration_tests_toolset::PayablePendingTx };
    let deposit_ident = if params.is_empty() || params.trailing_punct() {
        quote!(attached_deposit: u128)
    } else {
        quote!(,attached_deposit: u128)
    };

    let func = quote! {
        pub fn #name(&self, #params #deposit_ident) #return_ident {
            #serialize_args
            integration_tests_toolset::PayablePendingTx::new( String::from(#name_str), args, attached_deposit)
        }
    };

    func
}
