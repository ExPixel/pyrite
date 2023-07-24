use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, Data, DeriveInput, Expr, Ident, Lit, MetaList, Token, Type,
};

#[proc_macro_derive(IoRegister, attributes(field))]
pub fn io_register_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    try_io_register_macro(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn try_io_register_macro(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let Data::Struct(ref data)  = input.data else {
        return Err(syn::Error::new_spanned(input, "the IoRegister derive macro currently only supports structs"));
    };

    let mut maybe_value_field: Option<(&Ident, &Type)> = None;
    for field in data.fields.iter() {
        match &field.ident {
            Some(ref field_name) if maybe_value_field.is_none() && field_name == "value" => {
                maybe_value_field = Some((field_name, &field.ty));
            }

            _ => {
                return Err(syn::Error::new_spanned(
                    field,
                    "structs deriving IoRegister must have a single vield named `value`",
                ));
            }
        }
    }

    let Some((value_field_name, value_field_type)) = maybe_value_field else {
        return Err(syn::Error::new_spanned(
            input,
            "structs deriving IoRegister must have a single vield named `value`",
        ));
    };

    let mut field_attrs = input.attrs.iter().filter_map(|attr| {
        attr.meta
            .path()
            .get_ident()
            .filter(|ident| *ident == "field")?;

        match attr.meta.require_list() {
            Ok(attr) => Some(Ok(attr)),
            Err(err) => Some(Err(err)),
        }
    });

    let functions = std::iter::from_fn(|| match field_attrs.next()? {
        Ok(attr) => Some(get_fns_for_register_field(
            attr,
            value_field_name,
            value_field_type,
        )),
        Err(err) => {
            let compile_error = err.into_compile_error();
            Some(quote! { #compile_error })
        }
    });

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const fn new(value: #value_field_type) -> Self {
                Self { #value_field_name: value }
            }

            #(#functions)*
        }

        impl #impl_generics Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self::new(0)
            }
        }

        impl #impl_generics crate::memory::IoRegister<#value_field_type> for #name #ty_generics #where_clause {
            #[inline]
            fn read(self) -> #value_field_type {
                self.#value_field_name
            }

            #[inline]
            fn write(&mut self, value: #value_field_type) {
                self.#value_field_name = value;
            }
        }

        impl #impl_generics From<#value_field_type> for #name #ty_generics #where_clause {
            #[inline]
            fn from(value: #value_field_type) -> Self {
                Self { #value_field_name: value }
            }
        }

        impl #impl_generics From<#name #ty_generics> for #value_field_type #where_clause {
            #[inline]
            fn from(register: #name #ty_generics) -> Self {
                register.#value_field_name
            }
        }
    };

    // Hand the output tokens back to the compiler
    Ok(expanded)
}

fn get_fns_for_register_field(
    attr: &MetaList,
    value_field_name: &Ident,
    value_field_type: &Type,
) -> TokenStream {
    let arg: IoRegisterField = match attr.parse_args() {
        Ok(arg) => arg,
        Err(err) => {
            let compile_error = err.into_compile_error();
            return quote! { #compile_error };
        }
    };

    let field_getter = arg.name;
    let field_setter = Ident::new(&format!("set_{field_getter}"), field_getter.span());
    let field_type = arg.ty;

    match *arg.expr {
        Expr::Lit(literal) => match literal.lit {
            Lit::Int(int) => {
                quote! {
                    #[inline]
                    fn #field_getter(self) -> #field_type {
                        <#value_field_type as ::util::bits::BitOps>::get_bit(self.#value_field_name, #int) as #field_type
                    }

                    #[inline]
                    fn #field_setter(&mut self, value: #field_type) {
                        self.#value_field_name =
                            <#value_field_type as ::util::bits::BitOps>::put_bit(self.#value_field_name, #int, value);
                    }
                }
            }

            lit => {
                let err = syn::Error::new_spanned(lit, "literal field value must be an integer");
                let compile_error = err.into_compile_error();
                quote! { #compile_error }
            }
        },

        Expr::Range(range) => quote! {
            #[inline]
            fn #field_getter(self) -> #field_type {
                <#value_field_type as ::util::bits::BitOps>::get_bit_range(self.#value_field_name, #range) as #field_type
            }

            #[inline]
            fn #field_setter(&mut self, value: #field_type) {
                self.#value_field_name =
                    <#value_field_type as ::util::bits::BitOps>::put_bit_range(self.#value_field_name, #range, value as #value_field_type);
            }
        },

        Expr::Path(path) => quote! {
            #[inline]
            fn #field_getter(self) -> #field_type {
                self.#path
            }

            #[inline]
            fn #field_setter(&mut self, value: #field_type) {
                self.#path = value as #field_type;
            }
        },

        _ => {
            let err =
                syn::Error::new_spanned(arg.expr, "value of field must be a literal or range");
            let compile_error = err.into_compile_error();
            quote! { #compile_error }
        }
    }
}

#[derive(Debug)]
struct IoRegisterField {
    name: Ident,
    _colon_token: Token![:],
    ty: Type,
    _eq_token: Token![=],
    expr: Box<Expr>,
}

impl Parse for IoRegisterField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(IoRegisterField {
            name: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
            _eq_token: input.parse()?,
            expr: input.parse()?,
        })
    }
}
