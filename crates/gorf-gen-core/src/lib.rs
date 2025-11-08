use chumsky::Parser;
use gorf_core::types::GTerm;
use proc_macro2::TokenStream;
use std::convert::Infallible;
// use proc_macro::TokenStream;
use quasiquote::quasiquote;
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitStr};
pub mod codegen;
#[cfg(test)]
mod tests {
    use super::*;
}
