use crate::{
    has_attribute,
    types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo},
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

pub(crate) fn parse_struct_info(ast: ItemStruct) -> StructInfo {
    StructInfo {
        struct_name: ast.ident,
    }
}

pub(crate) fn parse_func_info(ast: ItemImpl) -> ImplInfo {
    let impl_ident = match *ast.self_ty.clone() {
        syn::Type::Path(path) => path
            .path
            .get_ident()
            // TODO: provide meaningful error message instead of just panicking
            .unwrap_or_else(|| panic!("{}", "ERROR IN PARSE"))
            .clone(),
        _ => Ident::new("", Span::call_site().into()),
    };
    let impl_name = format_ident!("{}Test", impl_ident);
    let mut func_infos: Vec<FunctionInfo> = vec![];

    for item in ast.items {
        match item {
            ImplItem::Method(method) => {
                if matches!(&method.vis, Visibility::Public(_)) || ast.trait_.is_some() {
                    parse_item_method(method)
                        .into_iter()
                        .for_each(|parsed_func_info| func_infos.push(parsed_func_info));
                }
            }
            _ => {}
        }
    }

    ImplInfo {
        impl_name,
        func_infos,
    }
}

fn parse_item_method(method: ImplItemMethod) -> Option<FunctionInfo> {
    let mut params_iter = method.sig.inputs.into_pairs().into_iter();
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
    } else {
        if let Some(first_arg) = params_iter.next() {
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
    }

    None
}

fn get_output(output: &ReturnType, handle_result: bool, is_init: bool) -> Type {
    let mut ret = parse_quote! {()};
    if !is_init {
        if let ReturnType::Type(_, ty) = output {
            ret = *ty.clone();
            if let Type::Path(tp) = &**ty {
                if let Some(path) = &tp.path.segments.first() {
                    if path.ident == "Result" && handle_result {
                        if let PathArguments::AngleBracketed(aba) = &path.arguments {
                            if let Some(ga) = aba.args.first() {
                                if let syn::GenericArgument::Type(ga_ty) = ga {
                                    ret = ga_ty.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    AccountIdReplace.visit_type_mut(&mut ret);
    ret
}

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

impl VisitMut for AccountIdReplace {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        match ty {
            Type::Array(type_arr) => self.visit_type_mut(type_arr.elem.as_mut()),
            Type::Path(type_path) => self.visit_type_path_mut(type_path),
            Type::Tuple(type_typle) => type_typle
                .elems
                .iter_mut()
                .for_each(|el| self.visit_type_mut(el)),
            _ => return,
        }
    }

    fn visit_type_path_mut(&mut self, ty: &mut TypePath) {
        if ty.path.is_ident("AccountId") {
            *ty = parse_quote!(workspaces::AccountId);
        } else if let Some(path_segment) = ty.path.segments.first_mut() {
            self.visit_path_segment_mut(path_segment);
        }
    }

    fn visit_path_segment_mut(&mut self, path_segment: &mut PathSegment) {
        if path_segment.ident.to_string().contains("Vec") {
            if let PathArguments::AngleBracketed(angl_bracketed) = &mut path_segment.arguments {
                if let Some(gen_arg) = angl_bracketed.args.first_mut() {
                    if let GenericArgument::Type(ty) = gen_arg {
                        self.visit_type_mut(ty);
                        return;
                    }
                }
            }
        }
    }
}
