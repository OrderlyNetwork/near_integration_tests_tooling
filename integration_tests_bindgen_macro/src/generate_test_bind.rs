use crate::types::{
    FunctionInfo, ImplInfo, IntegrationTestArguments, Mutability, Payable, StructInfo,
};
#[cfg(feature = "stress_test")]
use convert_case::{Case, Casing};
#[cfg(feature = "stress_test")]
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, token::Comma};
#[cfg(feature = "stress_test")]
use syn::{FnArg, Type};

pub(crate) fn generate_struct(input: TokenStream, struct_info: StructInfo) -> TokenStream {
    let name = format_ident!("{}Test", struct_info.struct_name);

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

pub(crate) fn generate_impl(
    input: TokenStream,
    impl_info: ImplInfo,
    #[allow(unused_variables)] args: &IntegrationTestArguments,
) -> TokenStream {
    let impl_name = impl_info.impl_name.clone();
    let mut func_stream_vec = vec![];
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

    #[cfg(feature = "stress_test")]
    {
        let mut func_operations = vec![];
        for func_info in impl_info.func_infos {
            func_operations.push(generate_operation(
                &func_info,
                &impl_info.struct_name,
                impl_info.impl_name.clone(),
                args,
            ));
        }

        let mut func_operations_output = quote! {
            #(#func_operations)*
        };
        func_operations_output.extend(input);

        func_output.extend(func_operations_output);
    }
    #[cfg(not(feature = "stress_test"))]
    {
        func_output.extend(input);
    }

    func_output
}

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
    let serialize_args = json_serialize(func_info);
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

#[cfg(feature = "stress_test")]
pub(crate) fn generate_operation(
    func_info: &FunctionInfo,
    struct_name: &str,
    impl_name: Ident,
    args: &IntegrationTestArguments,
) -> TokenStream {
    let func_name = func_info.function_name.clone();

    let name_str =
        struct_name.to_owned() + func_name.to_string().to_case(Case::UpperCamel).as_str();
    let name_camel_case = Ident::new(&name_str, func_info.function_name.span());

    let mut struct_params = TokenStream::new();
    for param in &func_info.params {
        let param = match param {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => match pat_type.ty.as_ref() {
                Type::Reference(type_ref) => {
                    let mut pat_type = pat_type.clone();
                    pat_type.ty = Box::new(type_ref.elem.as_ref().clone());
                    Some(pat_type)
                }
                _ => Some(pat_type.clone()),
            },
        };

        struct_params.extend(quote! {pub #param,});
    }

    let mut func_params = TokenStream::new();
    for param in &func_info.params {
        let param = match param {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => match pat_type.ty.as_ref() {
                _ => Some(pat_type.pat.clone()),
            },
        };

        func_params.extend(quote! {self.#param,});
    }

    let (struct_lifetime, static_lifetime) = (quote! {}, quote! {});
    // match func_info.mutability {
    //     Mutability::Mutable(Payable::Payable) => {
    //         struct_params
    //             .extend(quote! {pub caller: &'a workspaces::Account, pub attached_deposit: u128,});
    //         func_params.extend(quote! {self.caller, self.attached_deposit});
    //         (quote! {<'a>}, quote! {<'static>})
    //     }
    //     Mutability::Mutable(Payable::NonPayable) => {
    //         struct_params.extend(quote! {pub caller: &'a workspaces::Account,});
    //         func_params.extend(quote! {self.caller});
    //         (quote! {<'a>}, quote! {<'static>})
    //     }
    //     Mutability::Immutable => (quote! {}, quote! {}),
    // };

    let test_context = if args.internal {
        quote! {crate}
    } else {
        quote! {test_context}
    };

    quote! {
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        #[derive(Debug, Clone)]
        pub struct #name_camel_case #struct_lifetime{
            #struct_params
        }

        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        #[async_trait::async_trait]
        impl<U, const N: usize, const M: usize> #test_context::test_ops::runnable::Runnable<#impl_name, U, N, M> for #name_camel_case {
            async fn run_impl(&self, context: &#test_context::context::TestContext<#impl_name, U, N, M>)
            -> anyhow::Result<Option<integration_tests_toolset::statistic::statistic_consumer::Statistic>> {
                Ok(Some(integration_tests_toolset::statistic::statistic_consumer::Statistic::default()))
                // Ok(Some(context
                // .contract_controller
                // .get_template()
                // .#func_name(#func_params)
                // .await?.into()))
            }

            fn clone_dyn(&self) -> Box<dyn #test_context::test_ops::runnable::Runnable<#impl_name, U, N, M>> {
                Box::new(self.clone())
            }
        }

        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        impl<U, const N: usize, const M: usize> From<#name_camel_case #static_lifetime> for Box<dyn #test_context::test_ops::runnable::Runnable<#impl_name, U, N, M>> {
            fn from(op: #name_camel_case #static_lifetime) -> Self {
                Box::new(op)
            }
        }

        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        impl<U, const N: usize, const M: usize> From<#name_camel_case #static_lifetime> for #test_context::test_ops::block::Block<#impl_name, U, N, M> {
            fn from(op: #name_camel_case #static_lifetime) -> Self {
                Self {
                    chain: vec![Box::new(op)],
                    concurrent: vec![],
                }
            }
        }
    }
}
