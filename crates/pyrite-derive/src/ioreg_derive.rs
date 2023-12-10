use std::{cell::RefCell, ops::Range};

use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{
    parse::Parse, token::DotDotEq, Data, DeriveInput, Expr, ExprLit, ExprRange, GenericArgument,
    Ident, Lit, PathArguments, Token, Type, TypePath,
};
use util::bits::BitOps;

const PRIMITIVES: [(&str, u32); 11] = [
    ("i8", 8),
    ("u8", 9),
    ("i16", 16),
    ("u16", 16),
    ("i32", 32),
    ("u32", 32),
    ("i64", 64),
    ("u64", 64),
    ("i128", 128),
    ("u128", 128),
    ("bool", 1),
];

pub fn try_io_register_macro(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let Data::Struct(ref data) = input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "the IoRegister derive macro currently only supports structs",
        ));
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

    let value_field_bits = PRIMITIVES
        .into_iter()
        .find(|&(p, _)| {
            if p == "bool" {
                return false;
            }

            if let Type::Path(path) = value_field_type {
                matches!(path.path.get_ident(), Some(ident) if ident == p)
            } else {
                false
            }
        })
        .ok_or_else(|| {
            syn::Error::new_spanned(&input, "value field must be a primitive integer type")
        })?
        .1;

    let mut ioreg_fields = input.attrs.iter().filter_map(|attr| {
        attr.meta
            .path()
            .get_ident()
            .filter(|ident| *ident == "field")?;

        let attr = match attr.meta.require_list() {
            Ok(attr) => attr,
            Err(err) => return Some(Err(err)),
        };

        match attr.parse_args::<IoRegisterField>() {
            Ok(field) => Some(Ok(field)),
            Err(err) => Some(Err(err)),
        }
    });

    let r_bits = RefCell::new(u128::mask(value_field_bits));
    let w_bits = RefCell::new(u128::mask(value_field_bits));

    let functions = std::iter::from_fn(|| match ioreg_fields.next()? {
        Ok(field) => {
            if !field.flags.contains(IoRegisterFlags::READ) {
                let mut r_bits = r_bits.borrow_mut();
                *r_bits = r_bits.clear_bit_range(field.bit_range.clone());
            }

            if !field.flags.contains(IoRegisterFlags::WRITE) {
                let mut w_bits = w_bits.borrow_mut();
                *w_bits = w_bits.clear_bit_range(field.bit_range.clone());
            }

            let getter = field.getter(value_field_name, value_field_type);
            let setter = field.setter(value_field_name, value_field_type);
            Some(quote! { #getter #setter })
        }

        Err(err) => {
            let compile_error = err.into_compile_error();
            Some(quote! { #compile_error })
        }
    });

    let ioreg_read_fn = std::iter::once_with(|| {
        let read_bits = Literal::u128_unsuffixed(*r_bits.borrow());
        quote! {
            #[inline]
            fn read(self) -> #value_field_type {
                self.#value_field_name & #read_bits
            }
        }
    });

    let ioreg_write_fn = std::iter::once_with(|| {
        let write_bits = Literal::u128_unsuffixed(*w_bits.borrow());
        quote! {
            #[inline]
            fn write(&mut self, value: #value_field_type) {
                self.#value_field_name &= !#write_bits;
                self.#value_field_name |= value & #write_bits;
            }
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
            #(#ioreg_read_fn)*
            #(#ioreg_write_fn)*
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

struct IoRegisterField {
    range: ExprRange,
    bit_range: Range<u32>,
    flags: IoRegisterFlags,
    ty: Type,
    is_primitive: bool,
    is_bool: bool,
    name: Ident,
}

impl IoRegisterField {
    fn getter(&self, value_field_name: &Ident, value_field_type: &Type) -> TokenStream {
        let field_getter = &self.name;
        let field_type = &self.ty;
        let range = &self.range;

        if self.is_bool {
            quote! {
                #[inline]
                pub fn #field_getter(self) -> #field_type {
                    <#value_field_type as ::util::bits::BitOps>::get_bit_range(self.#value_field_name, #range) != 0
                }
            }
        } else if self.is_primitive {
            quote! {
                #[inline]
                pub fn #field_getter(self) -> #field_type {
                    <#value_field_type as ::util::bits::BitOps>::get_bit_range(self.#value_field_name, #range) as #field_type
                }
            }
        } else {
            quote! {
                #[inline]
                pub fn #field_getter(self) -> #field_type {
                    <#field_type as From<#value_field_type>>::from(
                        <#value_field_type as ::util::bits::BitOps>::get_bit_range(self.#value_field_name, #range)
                    )
                }
            }
        }
    }

    fn setter(&self, value_field_name: &Ident, value_field_type: &Type) -> TokenStream {
        let field_getter = &self.name;
        let field_setter = Ident::new(&format!("set_{field_getter}"), self.name.span());
        let field_type = &self.ty;
        let range = &self.range;

        if self.is_primitive {
            quote! {
                #[inline]
                pub fn #field_setter(&mut self, value: #field_type) {
                    self.#value_field_name =
                        <#value_field_type as ::util::bits::BitOps>::put_bit_range(self.#value_field_name, #range, value as #value_field_type);
                }
            }
        } else {
            quote! {
                #[inline]
                pub fn #field_setter(&mut self, value: #field_type) {
                    let value = <#value_field_type as From<#field_type>>::from(value);
                    self.#value_field_name =
                        <#value_field_type as ::util::bits::BitOps>::put_bit_range(self.#value_field_name, #range, value);
                }
            }
        }
    }

    fn extract_range_or_index(expr: Expr) -> syn::Result<(ExprRange, Range<u32>)> {
        match expr {
            start_end_expr @ Expr::Lit(ExprLit {
                lit: Lit::Int(..), ..
            }) => {
                let Expr::Lit(ExprLit {
                    lit: Lit::Int(ref int),
                    ..
                }) = start_end_expr
                else {
                    unreachable!()
                };
                let bit_range_int: u32 = int.base10_parse()?;
                let bit_range = bit_range_int..(bit_range_int + 1);
                let limits_inner = DotDotEq {
                    spans: [int.span(); 3],
                };
                let range = ExprRange {
                    attrs: Vec::new(),
                    start: Some(Box::new(start_end_expr.clone())),
                    limits: syn::RangeLimits::Closed(limits_inner),
                    end: Some(Box::new(start_end_expr)),
                };
                Ok((range, bit_range))
            }

            Expr::Range(
                range @ ExprRange {
                    start: Some(_),
                    end: Some(_),
                    ..
                },
            ) => {
                match (
                    range.start.as_deref().unwrap(),
                    range.end.as_deref().unwrap(),
                ) {
                    (
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(start),
                            ..
                        }),
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(end), ..
                        }),
                    ) => {
                        let bit_range_start: u32 = start.base10_parse()?;
                        let bit_range_end: u32 = end.base10_parse()?;
                        let bit_range = match &range.limits {
                            syn::RangeLimits::HalfOpen(_) => bit_range_start..bit_range_end,
                            syn::RangeLimits::Closed(_) => bit_range_start..(bit_range_end + 1),
                        };
                        Ok((range, bit_range))
                    }

                    _ => Err(syn::Error::new_spanned(
                        range,
                        "only closed/half-closed integer ranges allowed",
                    )),
                }
            }

            _ => todo!(),
        }
    }

    fn extract_type(base: Type, flags: &mut IoRegisterFlags) -> syn::Result<Type> {
        let is_flag_type_ident =
            |ident: &Ident| -> bool { ident == "readonly" || ident == "writeonly" };

        match base {
            Type::Path(TypePath { path, .. })
                if path.segments.len() == 1
                    && is_flag_type_ident(&path.segments.first().unwrap().ident)
                    && matches!(
                        &path.segments.first().unwrap().arguments,
                        PathArguments::AngleBracketed(args) if args.args.len() == 1 &&
                            matches!(args.args.first().unwrap(), GenericArgument::Type(..))
                    ) =>
            {
                let flag_type = path.segments.into_iter().next().unwrap();

                if flag_type.ident == "readonly" {
                    flags.remove(IoRegisterFlags::WRITE);
                } else if flag_type.ident == "writeonly" {
                    flags.remove(IoRegisterFlags::READ);
                } else {
                    unreachable!();
                }

                let PathArguments::AngleBracketed(args) = flag_type.arguments else {
                    unreachable!()
                };
                let GenericArgument::Type(ty) = args.args.into_iter().next().unwrap() else {
                    unreachable!()
                };
                Ok(ty)
            }
            base => Ok(base),
        }
    }
}

impl Parse for IoRegisterField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _colon: Token![:] = input.parse()?;
        let mut flags = IoRegisterFlags::ALL;
        let ty: Type = Self::extract_type(input.parse()?, &mut flags)?;
        let _equals: Token![=] = input.parse()?;
        let (range, bit_range): (ExprRange, Range<u32>) =
            Self::extract_range_or_index(input.parse()?)?;

        let mut is_primitive = false;
        let mut is_bool = false;

        if let Type::Path(ref path) = ty {
            if let Some(ident) = path.path.get_ident() {
                is_primitive = PRIMITIVES.into_iter().any(|(p, _)| ident == p);
                is_bool = ident == "bool";
            }
        }

        Ok(IoRegisterField {
            range,
            bit_range,
            flags,
            name,
            is_primitive,
            is_bool,
            ty,
        })
    }
}

bitflags::bitflags! {
    struct IoRegisterFlags: u8 {
        const READ = 0x1;
        const WRITE = 0x2;
        const ALL = Self::READ.bits() | Self::WRITE.bits();
    }
}
