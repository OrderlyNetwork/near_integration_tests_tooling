use proc_macro2::Ident;
use syn::{punctuated::Punctuated, FnArg, Token, Type};

#[derive(Debug)]
pub(crate) enum Mutability {
    Mutable(Payable),
    Immutable,
}

#[derive(Debug)]
pub(crate) enum Payable {
    Payable,
    NonPayable,
}

#[derive(Debug)]
pub(crate) struct ImplInfo {
    pub impl_name: Ident,
    pub func_infos: Vec<FunctionInfo>,
}

#[derive(Debug)]
pub(crate) struct FunctionInfo {
    pub function_name: Ident,
    pub params: Punctuated<FnArg, Token![,]>,
    pub params_ident: Vec<Ident>,
    pub mutability: Mutability,
    pub output: Type,
}

#[derive(Debug)]
pub(crate) struct StructInfo {
    pub struct_name: Ident,
}
