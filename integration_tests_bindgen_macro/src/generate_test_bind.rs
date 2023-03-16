// This module contains all stuff related to the generation of the contract bindings

use crate::types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, token::Comma};

// This function is used to generate a new struct with the name of the contract struct + Test
// ex. contract struct name is OracleConnector then the binding struct will have the name OracleConnectorTest.
pub(crate) fn generate_struct(input: TokenStream, struct_info: StructInfo) -> TokenStream {
    let name = format_ident!("{}Test", struct_info.struct_name);

    // generated struct will contain related workspaces::Contract in order to make call inside the generated contract functions
    let mut generated_struct: TokenStream = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        #[derive(Clone, Debug)]
        pub struct #name {
            pub contract: workspaces::Contract,
            pub measure_storage_usage: bool,
        }
    };

    generated_struct.extend(input.into_iter());
    generated_struct
}

// This func is used in cases when the integration_tests_bindgen macro specified for contract struct impl blocks
// and for trait implementation.
pub(crate) fn generate_impl(input: TokenStream, impl_info: ImplInfo) -> TokenStream {
    let impl_name = impl_info.impl_name.clone();
    let mut func_stream_vec = vec![];

    // In this loop we check the particular type of the smart contract function in order to generate the correct binding
    // There are 3 types of function Immutable, Mutable and Mutable::Payable depending on the type additional parameters
    // will be added to the smart-contract function arguments list
    for func_info in &impl_info.func_infos {
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
    let mut func_output = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        impl #impl_name {
            #(#func_stream_vec)*
        }
    };

    func_output.extend(input);

    func_output
}

// This function make a json serialized list of the parameters which later would be used in the call through the workspaces API
fn json_serialize(func_info: &FunctionInfo) -> TokenStream {
    let args: TokenStream = func_info
        .params_ident
        .iter()
        .fold(None, |acc: Option<TokenStream>, value| {
            let ident = value;
            let ident_str = ident.to_string();
            Some(match acc {
                None => quote! { #ident_str: #ident },
                Some(a) => quote! { #a, #ident_str: #ident },
            })
        })
        .unwrap_or_default(); // handle the case when their is no args

    quote! {
      let args = near_sdk::serde_json::json!({#args}).to_string().into_bytes();
    }
}

// This function is used to generate a binding for the Immutable(view) functions
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

// This function is used to generate a binding for the Mutable(call) functions
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

// This function is used to generate a binding for the Mutable(call) and payable function functions
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
    let serialize_args = json_serialize(func_info); // get the json representation of the arguments list
    let name = func_info.function_name.clone();
    let name_str = func_info.function_name.to_string();
    let mut params = func_info.params.clone();

    // added Comma to the end of the params list in case it is not empty or does not already contains it
    if !params.is_empty() && !params.trailing_punct() {
        params.push_punct(Comma::default());
    }

    let ty = func_info.output.output.clone();

    // Handle the case when the output type is PromiseOrValue<T> type
    let output = if func_info.output.is_promise {
        parse_quote! {Option<#ty>} // substitute to Option<T> instead of PromiseOrValue<T>
    } else {
        ty
    };

    let value = if output == parse_quote! {()} {
        quote! {()}
    } else if func_info.output.is_promise {
        // special case to handle PromiseOrValue<T> will return T
        quote! {#ret_type::value_from_res_for_promise(&res)?}
    } else {
        // in this handler we will try to deserialize to T
        quote! {#ret_type::value_from_res(&res)?}
    };

    let tx_call = quote! {
        // storage measurement before smart-contract function call
        let storage_usage_before = if self.measure_storage_usage { self.contract.view_account().await?.storage_usage } else { 0 };
        let res = #operation // function call will be executed here
        // storage measurement after the smart-contract function call
        let storage_usage = if self.measure_storage_usage { Some(self.contract.view_account().await?.storage_usage as i64 - storage_usage_before as i64) } else { None };

        // check the possible failures
        res.check_res_log_failures()?;
        // Convert to wrapped value which will also store call statistic
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
