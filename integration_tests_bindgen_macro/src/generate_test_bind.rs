use crate::types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, token::Comma};

pub(crate) fn generate_struct(input: TokenStream, struct_info: StructInfo) -> TokenStream {
    let name = format_ident!("{}Test", struct_info.struct_name);

    let mut generated_struct: TokenStream = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        #[derive(Clone, Debug)]
        pub struct #name {
            pub contract: workspaces::Contract,
            pub measure_storage_usage: bool,
        }
    }
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
                Payable::Payable => {
                    func_stream_vec.push(generate_payable_call_function(&func_info));
                }
                Payable::NonPayable => {
                    func_stream_vec.push(generate_non_payable_call_function(&func_info));
                }
            },
            Mutability::Immutable => {
                func_stream_vec.push(generate_view_function(&func_info));
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

pub(crate) fn generate_view_function(func_info: &FunctionInfo) -> TokenStream {
    let name_str = func_info.function_name.to_string();
    generate_function(
        func_info,
        quote! {integration_tests_toolset::pending_tx::immutable_tx::ImmutablePendingTx::new(&self.contract, String::from(#name_str), args).view().await?;},
        quote! {integration_tests_toolset::tx_result::ViewResult},
        quote! {},
        quote! {use integration_tests_toolset::pending_tx::view::View;},
    )
}

pub(crate) fn generate_non_payable_call_function(func_info: &FunctionInfo) -> TokenStream {
    let name_str = func_info.function_name.to_string();
    generate_function(
        func_info,
        quote! {integration_tests_toolset::pending_tx::mutable_tx::MutablePendingTx::new(&self.contract, String::from(#name_str), args).call(caller).await?;},
        quote! {integration_tests_toolset::tx_result::CallResult},
        quote! {caller: &workspaces::Account},
        quote! {use integration_tests_toolset::pending_tx::call::Call;},
    )
}

pub(crate) fn generate_payable_call_function(func_info: &FunctionInfo) -> TokenStream {
    let name_str = func_info.function_name.to_string();
    generate_function(
        func_info,
        quote! {integration_tests_toolset::pending_tx::payable_tx::PayablePendingTx::new(&self.contract, String::from(#name_str), args, attached_deposit).call(caller).await?;},
        quote! {integration_tests_toolset::tx_result::CallResult},
        quote! {caller: &workspaces::Account, attached_deposit: u128},
        quote! {use integration_tests_toolset::pending_tx::call::Call;},
    )
}

pub(crate) fn generate_function(
    func_info: &FunctionInfo,
    operation: TokenStream,
    ret_type: TokenStream,
    additional_params: TokenStream,
    use_tx_trait: TokenStream,
) -> TokenStream {
    let serialize_args = json_serialize(&func_info);
    let name = func_info.function_name.clone();
    let name_str = func_info.function_name.to_string();
    let mut params = func_info.params.clone();
    if !params.is_empty() && !params.trailing_punct() {
        params.push_punct(Comma::default());
    }
    let output = func_info.output.clone();

    let value = if output == parse_quote! {()} {
        quote! {()}
    } else {
        quote! {#ret_type::value_from_res(&res)?}
    };

    let tx_call = quote! {
        let storage_usage_before = if self.measure_storage_usage { self.contract.view_account().await?.storage_usage } else { 0 };
        let res = #operation
        let storage_usage = if self.measure_storage_usage { Some(self.contract.view_account().await?.storage_usage as i64 - storage_usage_before as i64) } else { None };

        res.check_res_log_failures()?;
        #ret_type::from_res(#name_str.to_owned(), #value, storage_usage, res)
    };

    quote! {
        pub async fn #name(&self, #params #additional_params) -> integration_tests_toolset::error::Result<integration_tests_toolset::tx_result::TxResult<#output>> {
            use integration_tests_toolset::{tx_result::FromRes, res_logger::ResLogger};
            #use_tx_trait
            #serialize_args
            #tx_call
        }
    }
}
