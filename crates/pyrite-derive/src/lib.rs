mod ioreg_derive;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IoRegister, attributes(field))]
pub fn io_register_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    ioreg_derive::try_io_register_macro(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
