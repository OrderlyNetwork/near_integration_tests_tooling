use proc_macro2::Ident;
use syn::{punctuated::Punctuated, FnArg, Token, Type};

// Store the information about the method mutability
#[derive(Debug)]
pub(crate) enum Mutability {
    Mutable(Payable),
    Immutable,
}

// Extend the Mutability by enum by classifying whether the method is payable or not
#[derive(Debug)]
pub(crate) enum Payable {
    Payable,
    NonPayable,
}

// Struct for the info required to generate the impl block
#[derive(Debug)]
pub(crate) struct ImplInfo {
    #[allow(dead_code)]
    pub struct_name: String,
    pub impl_name: Ident,
    pub func_infos: Vec<FunctionInfo>,
}

// Stores the data required for the smart contract function output representation
#[derive(Debug)]
pub(crate) struct OutputType {
    pub output: Type,
    pub is_promise: bool,
}

// Stores the data required for the function generation
#[derive(Debug)]
pub(crate) struct FunctionInfo {
    pub function_name: Ident,
    pub params: Punctuated<FnArg, Token![,]>,
    pub params_ident: Vec<Ident>,
    pub mutability: Mutability,
    pub output: OutputType,
}

#[derive(Debug)]
pub(crate) struct StructInfo {
    pub struct_name: Ident,
}
