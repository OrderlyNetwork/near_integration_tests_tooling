use crate::{
    has_attribute,
    types::{FunctionInfo, ImplInfo, Mutability, OutputType, Payable, StructInfo},
};
use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::{
    parse_quote,
    punctuated::{IntoPairs, Punctuated},
    token::Comma,
    visit_mut::VisitMut,
    FnArg, GenericArgument, ImplItem, ImplItemMethod, ItemImpl, ItemStruct, Pat, PathArguments,
    PathSegment, ReturnType, Type, TypePath, Visibility,
};

// Used to get the name of the contract struct
pub(crate) fn parse_struct_info(ast: ItemStruct) -> StructInfo {
    StructInfo {
        struct_name: ast.ident,
    }
}

// TODO: parse macro like
// near_contract_standards::impl_fungible_token_core!(Contract, token);
// Note: it unwraps in several #[near_bindgen] impls

// Used to parse the required info from the function contract signature
pub(crate) fn parse_func_info(ast: ItemImpl) -> ImplInfo {
    // extracting Impl block name
    let impl_ident = match *ast.self_ty.clone() {
        syn::Type::Path(path) => path
            .path
            .get_ident()
            // TODO: provide meaningful error message instead of just panicking
            .unwrap_or_else(|| panic!("{}", "ERROR IN PARSE"))
            .clone(),
        _ => Ident::new("", Span::call_site()),
    };
    let mut func_infos: Vec<FunctionInfo> = vec![];

    // Extracting function info for every particular function in the impl block
    for item in ast.items {
        if let ImplItem::Method(method) = item {
            // parse only public functions or defined in trait impl block because they also are public
            if matches!(&method.vis, Visibility::Public(_)) || ast.trait_.is_some() {
                parse_item_method(method)
                    .into_iter()
                    .for_each(|parsed_func_info| func_infos.push(parsed_func_info));
            }
        }
    }

    ImplInfo {
        struct_name: impl_ident.to_string(),
        impl_name: format_ident!("{}Test", impl_ident), // the name of the impl block would be extended with Test
        func_infos,
    }
}

// Parse the smart-contract method signature to extract relevant info into the FunctionInfo
fn parse_item_method(method: ImplItemMethod) -> Option<FunctionInfo> {
    let mut params_iter = method.sig.inputs.into_pairs();
    // check wether method has marked with the init attribute
    let is_init = has_attribute(&method.attrs, "init");

    // TODO: refactor code below, extract repeated code (they are not identical!)
    if is_init {
        return Some(FunctionInfo {
            function_name: method.sig.ident,
            params: get_params(&params_iter),
            params_ident: get_idents(&params_iter),
            mutability: Mutability::Mutable(if has_attribute(&method.attrs, "payable") {
                Payable::Payable
            } else {
                Payable::NonPayable
            }),
            output: get_output(
                &method.sig.output,
                has_attribute(&method.attrs, "handle_result"),
                is_init,
            ),
        });
    } else if let Some(first_arg) = params_iter.next() {
        // check if the first argument is self
        if let FnArg::Receiver(self_value) = first_arg.value() {
            return Some(FunctionInfo {
                function_name: method.sig.ident,
                params: get_params(&params_iter),
                params_ident: get_idents(&params_iter),
                mutability: self_value.mutability.map_or(Mutability::Immutable, |_| {
                    Mutability::Mutable(if has_attribute(&method.attrs, "payable") {
                        Payable::Payable
                    } else {
                        Payable::NonPayable
                    })
                }),
                output: get_output(
                    &method.sig.output,
                    has_attribute(&method.attrs, "handle_result"),
                    is_init,
                ),
            });
        }
    }

    None
}

// Parse the output type for the generated function
fn get_output(output: &ReturnType, handle_result: bool, is_init: bool) -> OutputType {
    let mut ret = parse_quote! {()};
    let mut is_promise = false;
    if !is_init {
        if let ReturnType::Type(_, ty) = output {
            ret = *ty.clone();
            if let Type::Path(tp) = &**ty {
                if let Some(path) = &tp.path.segments.first() {
                    is_promise = path.ident == "PromiseOrValue";
                    if (path.ident == "Result" && handle_result) || is_promise {
                        if let PathArguments::AngleBracketed(aba) = &path.arguments {
                            if let Some(syn::GenericArgument::Type(ga_ty)) = aba.args.first() {
                                ret = ga_ty.clone();
                            }
                        }
                    }
                }
            }
        }
    }
    AccountIdReplace.visit_type_mut(&mut ret);
    OutputType {
        output: ret,
        is_promise,
    }
}

// Transform IntoPairs to the Punctuated which then should be used for the generation
fn get_params(params_iter: &IntoPairs<FnArg, Comma>) -> Punctuated<FnArg, Comma> {
    Punctuated::from_iter(
        params_iter
            .clone()
            .into_iter()
            .map(|mut el| match el.value_mut() {
                FnArg::Receiver(_) => el,
                FnArg::Typed(pat_type) => {
                    AccountIdReplace.visit_type_mut(pat_type.ty.as_mut());
                    el
                }
            }),
    )
}

// Get ident(the name) from the parameter list
fn get_idents(params_iter: &IntoPairs<FnArg, Comma>) -> Vec<Ident> {
    params_iter
        .clone()
        .map(|el| el.value().clone())
        .filter_map(|el| match el {
            FnArg::Typed(pat) => match *pat.pat {
                Pat::Ident(ident) => Some(ident.ident),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

struct AccountIdReplace;

// This visitor implementation is used to substitute near_sdk::AccountId to workspaces::AccountId in parameters or return type
// This implementation works recursively even for such complicated structures as Vector<Vector<Option<AccountId>>>
impl VisitMut for AccountIdReplace {
    // visitor function for the type(which is the entry point)
    fn visit_type_mut(&mut self, ty: &mut Type) {
        match ty {
            Type::Array(type_arr) => self.visit_type_mut(type_arr.elem.as_mut()),
            Type::Path(type_path) => self.visit_type_path_mut(type_path),
            Type::Tuple(type_tuple) => type_tuple
                .elems
                .iter_mut()
                .for_each(|el| self.visit_type_mut(el)),
            Type::Reference(type_ref) => self.visit_type_mut(type_ref.elem.as_mut()),
            _ => (),
        }
    }

    // visitor for the TypePath(which could be the end point)
    fn visit_type_path_mut(&mut self, ty: &mut TypePath) {
        if ty.path.is_ident("AccountId") {
            *ty = parse_quote!(workspaces::AccountId);
        } else if let Some(path_segment) = ty.path.segments.first_mut() {
            self.visit_path_segment_mut(path_segment);
        }
    }

    // visitor for path segment(which is basically a generic type like Vec<>, Option<> and could be nested like Vec<Vec<>>)
    fn visit_path_segment_mut(&mut self, path_segment: &mut PathSegment) {
        if path_segment.ident.to_string().contains("Vec")
            || path_segment.ident.to_string().contains("Option")
        {
            if let PathArguments::AngleBracketed(angl_bracketed) = &mut path_segment.arguments {
                if let Some(GenericArgument::Type(ty)) = angl_bracketed.args.first_mut() {
                    self.visit_type_mut(ty);
                }
            }
        }
    }
}
