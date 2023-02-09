use crate::types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, token::Comma};

pub(crate) fn generate_struct(input: TokenStream, struct_info: StructInfo) -> TokenStream {
    let name = format_ident!("{}Test", struct_info.struct_name);

    let mut generated_struct: TokenStream = quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        #[derive(Clone)]
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
        quote! {let res = integration_tests_toolset::ImmutablePendingTx::new(&self.contract, String::from(#name_str), args).view().await?;},
        quote! {integration_tests_toolset::ViewResult},
        quote! {},
        quote! {use integration_tests_toolset::View;},
    )
}

pub(crate) fn generate_non_payable_call_function(func_info: &FunctionInfo) -> TokenStream {
    let name_str = func_info.function_name.to_string();
    generate_function(
        func_info,
        quote! {let res = integration_tests_toolset::MutablePendingTx::new(&self.contract, String::from(#name_str), args).call(caller).await?.into_result()?;},
        quote! {integration_tests_toolset::CallResult},
        quote! {caller: &workspaces::Account},
        quote! {use integration_tests_toolset::Call;},
    )
}

pub(crate) fn generate_payable_call_function(func_info: &FunctionInfo) -> TokenStream {
    let name_str = func_info.function_name.to_string();
    generate_function(
        func_info,
        quote! {let res = integration_tests_toolset::PayablePendingTx::new(&self.contract, String::from(#name_str), args, attached_deposit).call(caller).await?.into_result()?;},
        quote! {integration_tests_toolset::CallResult},
        quote! {caller: &workspaces::Account, attached_deposit: u128},
        quote! {use integration_tests_toolset::Call;},
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
    let mut params = func_info.params.clone();
    if !params.is_empty() && !params.trailing_punct() {
        params.push_punct(Comma::default());
    }
    let output = func_info.output.clone();

    let operation_with_storage_measure = quote! {
        let (res, storage_usage) = if self.measure_storage_usage {
            let storage_usage_before = self.contract.view_account().await?.storage_usage;
            #operation
            let storage_usage_after = self.contract.view_account().await?.storage_usage;
            (res, Some(storage_usage_after as i64 - storage_usage_before as i64))
        } else {
            #operation
            (res, None)
        };
    };

    let value = if output == parse_quote! {()} {
        quote! {()}
    } else {
        quote! {res.json()?}
    };

    let tx_call = quote! {
        #operation_with_storage_measure
        // For mutable functions:
        // TODO: check res.receipt_failures() for errors, rise TestError::Receipt(String) if there are any
        // TODO:: check res.receipt_outcomes() for logs
        Ok(#ret_type{value: #value, res, storage_usage})
    };

    quote! {
        pub async fn #name(&self, #params #additional_params) -> integration_tests_toolset::Result<#ret_type<#output>> {
            #use_tx_trait
            #serialize_args
            #tx_call
        }
    }
}
