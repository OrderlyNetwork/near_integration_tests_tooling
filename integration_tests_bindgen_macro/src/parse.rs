use crate::types::{FunctionInfo, ImplInfo, Mutability, Payable, StructInfo};
use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::{
    parse_quote,
    punctuated::{Pair, Punctuated},
    visit_mut::VisitMut,
    FnArg, ImplItem, ImplItemMethod, ItemImpl, ItemStruct, Pat, PatType, Type, TypePath,
    Visibility,
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

    if let Some(first_arg) = params_iter.next() {
        if let FnArg::Receiver(self_value) = first_arg.value() {
            let is_payable =
                method
                    .attrs
                    .iter()
                    .map(|attr| attr.parse_meta())
                    .any(|res| match res {
                        Ok(meta) => meta
                            .path()
                            .get_ident()
                            .map(|el| el.to_string())
                            .filter(|el| *el == String::from("payable"))
                            .is_some(),
                        Err(_) => false,
                    });

            let mutability = self_value.mutability.map_or(Mutability::Immutable, |_| {
                Mutability::Mutable(if is_payable {
                    Payable::Payable
                } else {
                    Payable::NonPayable
                })
            });

            let params_ident = params_iter
                .clone()
                .map(|el| el.value().clone())
                .filter_map(|el| match el {
                    FnArg::Typed(pat) => match *pat.pat {
                        Pat::Ident(ident) => Some(ident.ident),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();

            let replaced_params = Punctuated::from_iter(params_iter.map(|el| match el.value() {
                FnArg::Receiver(_) => el,
                FnArg::Typed(PatType {
                    attrs,
                    pat,
                    colon_token,
                    ty,
                }) => {
                    if let Type::Path(ty) = &**ty {
                        if ty.path.is_ident("AccountId") {
                            let mut type_path = ty.clone();
                            AccountIdReplace.visit_type_path_mut(&mut type_path);
                            let res = FnArg::Typed(PatType {
                                attrs: attrs.clone(),
                                pat: pat.clone(),
                                colon_token: colon_token.clone(),
                                ty: Box::from(Type::Path(type_path)),
                            });
                            return Pair::new(res, el.punct().cloned());
                        }
                        el
                    } else {
                        el
                    }
                }
            }));

            return Some(FunctionInfo {
                function_name: method.sig.ident,
                params: replaced_params,
                params_ident,
                mutability,
            });
        }
    }
    None
}

struct AccountIdReplace;

impl VisitMut for AccountIdReplace {
    fn visit_type_path_mut(&mut self, type_path: &mut TypePath) {
        *type_path = parse_quote!(workspaces::AccountId);
    }
}
