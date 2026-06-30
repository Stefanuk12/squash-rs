use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument,
    GenericParam, Lifetime, LifetimeParam, PathArguments, Type,
};

/// How a record field participates in v5's record encoding.
enum FieldKind {
    /// Plain field, serialized via its `SquashObject` impl in sorted order.
    Regular,
    /// `bool` — packed into the bool bitarray.
    Bool,
    /// `Option<bool>` — value packed into the bool bitarray, presence into flags.
    OptBool,
    /// `Option<T>` (T != bool) — value serialized when present, presence into flags.
    Opt,
}

fn type_last_ident(ty: &Type) -> Option<String> {
    if let Type::Path(p) = ty {
        p.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

fn is_bool(ty: &Type) -> bool {
    type_last_ident(ty).as_deref() == Some("bool")
}

/// Inner `T` if `ty` is `Option<T>`.
fn option_inner(ty: &Type) -> Option<Type> {
    if let Type::Path(p) = ty {
        let seg = p.path.segments.last()?;
        if seg.ident != "Option" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &seg.arguments {
            if let Some(GenericArgument::Type(inner)) = args.args.first() {
                return Some(inner.clone());
            }
        }
    }
    None
}

fn classify(ty: &Type) -> FieldKind {
    if is_bool(ty) {
        FieldKind::Bool
    } else if let Some(inner) = option_inner(ty) {
        if is_bool(&inner) {
            FieldKind::OptBool
        } else {
            FieldKind::Opt
        }
    } else {
        FieldKind::Regular
    }
}

fn reverse_deserialize_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let name = &input.ident;

    let has_generics = !input.generics.params.is_empty();

    let mut generics = input.generics.clone();
    let lt = Lifetime::new("'de", Span::call_site());
    generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(lt.clone())));

    let (impl_generics, _, _) = generics.split_for_impl();

    let (_, ty_generics_2, where_clause_2) = input.generics.split_for_impl();
    let where_clause_2 = if where_clause_2.is_some() {
        quote! {
            #where_clause_2 + ::serde::Deserialize<'de>
        }
    } else {
        quote! {}
    };

    let (field_idents, field_types): (Vec<_>, Vec<_>) = if let Fields::Named(fields) = &data.fields
    {
        fields
            .named
            .iter()
            .map(|f| (f.ident.as_ref().unwrap(), &f.ty))
            .rev()
            .unzip()
    } else {
        panic!("ReverseDeserialize can only be derived for structs with named fields");
    };

    let field_names_string = field_idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>();
    let field_enum = field_idents
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("Field{}", i))
        .collect::<Vec<_>>();

    let (main_visitor, init_visitor) = if has_generics {
        (
            quote! {
                struct MainVisitor #ty_generics_2(::core::marker::PhantomData #ty_generics_2);
            },
            quote! {
                MainVisitor(::core::marker::PhantomData)
            },
        )
    } else {
        (
            quote! {
                struct MainVisitor;
            },
            quote! {
                MainVisitor
            },
        )
    };

    let expanded = quote! {
        impl #impl_generics ::serde::Deserialize<'de> for #name #ty_generics_2 #where_clause_2 {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                enum Field {
                    #(#field_enum),*
                }

                #main_visitor
                impl #impl_generics ::serde::de::Visitor<'de> for MainVisitor #ty_generics_2 #where_clause_2 {
                    type Value = #name #ty_generics_2;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(stringify!("struct {}", #name))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        #(
                            let #field_idents = seq.next_element::<#field_types>()?.ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        )*

                        Ok(Self::Value {
                            #(#field_idents),*
                        })
                    }
                }

                const FIELDS: &[&str] = &[#(#field_names_string),*];
                deserializer.deserialize_struct(stringify!(#name), FIELDS, #init_visitor)
            }
        }
    };

    TokenStream::from(expanded)
}

fn reverse_deserialize_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;

    // The variant tag is read/written as a single byte; >256 variants would alias
    // (see `derive_squash_object_enum`). Reject at compile time rather than truncate.
    if data.variants.len() > 256 {
        return syn::Error::new_spanned(
            &input.ident,
            "ReverseDeserialize enums support at most 256 variants (the variant tag is a single byte)",
        )
        .to_compile_error()
        .into();
    }

    let has_generics = !input.generics.params.is_empty();

    let mut generics = input.generics.clone();
    let lt = Lifetime::new("'de", Span::call_site());
    generics
        .params
        .push(GenericParam::Lifetime(LifetimeParam::new(lt.clone())));

    let (impl_generics, _, _) = generics.split_for_impl();

    let (_, ty_generics_2, where_clause_2) = input.generics.split_for_impl();
    let where_clause_2 = if where_clause_2.is_some() {
        quote! {
            #where_clause_2 + ::serde::Deserialize<'de>
        }
    } else {
        quote! {}
    };

    let (field_index, field_enum) = data.variants.iter().enumerate().fold(
        (Vec::new(), Vec::new()),
        |(mut field_index, mut field_enum), (i, variant)| {
            let ident = &variant.ident;
            field_index.push(i as u8);
            field_enum.push(ident.clone());
            (field_index, field_enum)
        },
    );

    let (main_visitor, init_visitor) = if has_generics {
        (
            quote! {
                struct MainVisitor #ty_generics_2(::core::marker::PhantomData #ty_generics_2);
            },
            quote! {
                MainVisitor(::core::marker::PhantomData)
            },
        )
    } else {
        (
            quote! {
                struct MainVisitor;
            },
            quote! {
                MainVisitor
            },
        )
    };

    let expanded = quote! {
        impl #impl_generics ::serde::Deserialize<'de> for #name #ty_generics_2 #where_clause_2 {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                #[derive(::serde::Deserialize)]
                #[serde(field_identifier, rename_all = "lowercase")]
                enum Field { C, T }

                #main_visitor
                impl #impl_generics ::serde::de::Visitor<'de> for MainVisitor #ty_generics_2 #where_clause_2 {
                    type Value = #name #ty_generics_2;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(stringify!("enum {}", #name))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                    where
                        A: ::serde::de::SeqAccess<'de>,
                    {
                        let tag = seq.next_element::<u8>()?.ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?;
                        match tag {
                            #(
                                #field_index => Ok(#name::#field_enum(seq.next_element::<#field_enum>()?.ok_or_else(|| ::serde::de::Error::invalid_length(0, &self))?)),
                            )*
                            _ => Err(::serde::de::Error::invalid_length(0, &self)),
                        }
                    }
                }

                const FIELDS: &[&str] = &["c", "t"];
                deserializer.deserialize_struct(stringify!(#name), FIELDS, #init_visitor)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ReverseDeserialize)]
pub fn reverse_deserialize_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    if let Data::Struct(data) = &input.data {
        reverse_deserialize_struct(&input, data)
    } else if let Data::Enum(data) = &input.data {
        reverse_deserialize_enum(&input, data)
    } else {
        panic!("ReverseDeserialize can only be derived for structs or enums");
    }
}

fn derive_squash_object_struct(
    input: &DeriveInput,
    data: &DataStruct,
    squash_object: proc_macro2::TokenStream,
    squash_cursor: proc_macro2::TokenStream,
    result: proc_macro2::TokenStream,
) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = if let Fields::Named(fields) = &data.fields {
        fields.named.iter().collect::<Vec<_>>()
    } else {
        panic!("SquashObject can only be derived for structs with named fields");
    };

    let mut regular: Vec<&syn::Ident> = Vec::new();
    let mut bools: Vec<&syn::Ident> = Vec::new();
    let mut optbools: Vec<&syn::Ident> = Vec::new();
    let mut opts: Vec<&syn::Ident> = Vec::new();
    for f in &fields {
        let ident = f.ident.as_ref().unwrap();
        match classify(&f.ty) {
            FieldKind::Bool => bools.push(ident),
            FieldKind::OptBool => optbools.push(ident),
            FieldKind::Opt => opts.push(ident),
            FieldKind::Regular => regular.push(ident),
        }
    }
    let by_name = |a: &&syn::Ident, b: &&syn::Ident| a.to_string().cmp(&b.to_string());
    regular.sort_by(by_name);
    bools.sort_by(by_name);
    optbools.sort_by(by_name);
    opts.sort_by(by_name);

    let bool_count = bools.len();
    let optbool_count = optbools.len();
    let opt_total = optbool_count + opts.len();

    let push_regular = regular.iter().map(|id| quote! {
        __count += __cursor.push(self.#id)?;
    });
    let bools_init = bools.iter().map(|id| quote! { self.#id });
    let push_optbool = optbools.iter().map(|id| quote! {
        match self.#id {
            ::core::option::Option::Some(__v) => { __bools.push(__v); __flags.push(true); }
            ::core::option::Option::None => { __flags.push(false); }
        }
    });
    let push_opt = opts.iter().map(|id| quote! {
        match self.#id {
            ::core::option::Option::Some(__v) => { __count += __cursor.push(__v)?; __flags.push(true); }
            ::core::option::Option::None => { __flags.push(false); }
        }
    });

    // ---- pop (mirror, reading the LIFO stream back-to-front) ----
    let bool_assign = bools.iter().enumerate().map(|(i, id)| {
        let idx = proc_macro2::Literal::usize_unsuffixed(i);
        quote! { let #id = __bools[#idx]; }
    });
    let optbool_rev = optbools.iter().enumerate().rev().map(|(i, id)| {
        let fi = proc_macro2::Literal::usize_unsuffixed(i);
        quote! {
            let #id = if __flags[#fi] { __b -= 1; ::core::option::Option::Some(__bools[__b]) }
                      else { ::core::option::Option::None };
        }
    });
    let opt_rev = opts.iter().enumerate().rev().map(|(i, id)| {
        let fi = proc_macro2::Literal::usize_unsuffixed(i + optbool_count);
        quote! {
            let #id = if __flags[#fi] { ::core::option::Option::Some(__cursor.pop()?) }
                      else { ::core::option::Option::None };
        }
    });
    let regular_rev = regular.iter().rev().map(|id| quote! { let #id = __cursor.pop()?; });

    let all_idents: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();

    let expanded = quote! {
        impl #impl_generics #squash_object for #name #ty_generics #where_clause {
            #[allow(unused_mut, unused_variables, clippy::reversed_empty_ranges)]
            fn pop_obj<Obj>(__cursor: &mut Obj) -> #result<Self>
            where
                Obj: #squash_cursor,
                Self: Sized {
                let __flags = ::squash::codec::pop_bits(__cursor, #opt_total)?;
                let mut __actual_bool_count = #bool_count;
                for __i in 0..#optbool_count { if __flags[__i] { __actual_bool_count += 1; } }
                let __bools = ::squash::codec::pop_bits(__cursor, __actual_bool_count)?;
                let mut __b = __actual_bool_count;
                #(#bool_assign)*
                #(#optbool_rev)*
                #(#opt_rev)*
                #(#regular_rev)*
                Ok(#name { #(#all_idents),* })
            }

            #[allow(unused_mut, unused_variables)]
            fn push_obj<Obj: #squash_cursor>(self, __cursor: &mut Obj) -> #result<usize> {
                let mut __count = 0;
                #(#push_regular)*
                let mut __bools: ::std::vec::Vec<bool> = ::std::vec![ #(#bools_init),* ];
                let mut __flags: ::std::vec::Vec<bool> = ::std::vec::Vec::new();
                #(#push_optbool)*
                #(#push_opt)*
                __count += ::squash::codec::push_bits(__cursor, &__bools)?;
                __count += ::squash::codec::push_bits(__cursor, &__flags)?;
                Ok(__count)
            }
        }
    };

    TokenStream::from(expanded)
}

fn derive_squash_object_enum(
    input: &DeriveInput,
    data: &DataEnum,
    squash_object: proc_macro2::TokenStream,
    squash_cursor: proc_macro2::TokenStream,
    result: proc_macro2::TokenStream,
) -> TokenStream {
    let name = &input.ident;

    // The variant tag is a single byte (`#i as u8`), so a 257th variant would
    // alias tag 0 and silently decode as the wrong variant. Reject at compile time
    // rather than truncate.
    if data.variants.len() > 256 {
        return syn::Error::new_spanned(
            &input.ident,
            "SquashObject enums support at most 256 variants (the variant tag is a single byte)",
        )
        .to_compile_error()
        .into();
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (field_pop, field_push) = data.variants.iter().enumerate().fold(
        (Vec::new(), Vec::new()),
        |(mut field_pop, mut field_push), (i, variant)| {
            let ident = &variant.ident;
            let i = i as u8;
            match &variant.fields {
                syn::Fields::Unit => {
                    field_pop.push(quote! {
                        #i => Ok(#name::#ident),
                    });
                    field_push.push(quote! {
                        #name::#ident => {
                            __count += __cursor.push(#i as u8)?;
                        }
                    });
                }
                syn::Fields::Unnamed(_) => {
                    field_pop.push(quote! {
                        #i => Ok(#name::#ident(__cursor.pop()?)),
                    });
                    field_push.push(quote! {
                        #name::#ident(__value) => {
                            __count += __cursor.push(__value)?;
                            __count += __cursor.push(#i as u8)?;
                        }
                    });
                },
                syn::Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    let field_names_reversed = field_names.iter().rev();

                    field_pop.push(quote! {
                        #i => Ok(#name::#ident {
                            #(#field_names_reversed: __cursor.pop()?,)*
                        }),
                    });
                    field_push.push(quote! {
                        #name::#ident { #(#field_names),* } => {
                            #(
                                __count += __cursor.push(#field_names)?;
                            )*
                            __count += __cursor.push(#i as u8)?;
                        }
                    });
                }
            };
            (field_pop, field_push)
        },
    );

    let expanded = quote! {
        impl #impl_generics #squash_object for #name #ty_generics #where_clause {
            fn pop_obj<Obj>(__cursor: &mut Obj) -> #result<Self>
            where
                Obj: #squash_cursor,
                Self: Sized {
                let tag = __cursor.pop::<u8>()?;
                match tag {
                    #(#field_pop)*
                    _ => Err(::squash::Error::DeserializeVariantNotMatched),
                }
            }

            fn push_obj<Obj: #squash_cursor>(self, __cursor: &mut Obj) -> #result<usize> {
                let mut __count = 0;
                match self {
                    #(
                        #field_push
                    )*
                }
                Ok(__count)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(SquashObject)]
pub fn derive_squash_object(input: TokenStream) -> TokenStream {
    let (squash_object, squash_cursor, result) =(
        quote!(::squash::SquashObject),
        quote!(::squash::SquashCursor),
        quote!(::squash::Result),
    );

    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = &input.data {
        derive_squash_object_struct(&input, data, squash_object, squash_cursor, result)
    } else if let Data::Enum(data) = &input.data {
        derive_squash_object_enum(&input, data, squash_object, squash_cursor, result)
    } else {
        panic!("SquashObject can only be derived for structs");
    }
}
