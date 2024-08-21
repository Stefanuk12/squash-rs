use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, GenericParam,
    Lifetime, LifetimeParam,
};

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

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_pushes: Vec<_> = field_names
        .iter()
        .map(|name| quote! { count += cursor.push(self.#name.clone())?; })
        .collect();
    let field_pops: Vec<_> = field_names
        .iter()
        .map(|name| quote! { #name: cursor.pop()?, })
        .collect();
    let field_pops_rev = field_pops.iter().rev();

    let expanded = quote! {
        impl #impl_generics #squash_object for #name #ty_generics #where_clause {
            fn pop_obj<Obj>(cursor: &mut Obj) -> #result<Self>
            where
                Obj: #squash_cursor,
                Self: Sized {
                Ok(#name {
                    #(#field_pops_rev)*
                })
            }

            fn push_obj<Obj: #squash_cursor>(self, cursor: &mut Obj) -> #result<usize> {
                let mut count = 0;
                #(#field_pushes)*
                Ok(count)
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
                            count += cursor.push(#i as u8)?;
                        }
                    });
                }
                syn::Fields::Unnamed(_) => {
                    field_pop.push(quote! {
                        #i => Ok(#name::#ident(cursor.pop()?)),
                    });
                    field_push.push(quote! {
                        #name::#ident(value) => {
                            count += cursor.push(#i as u8)?;
                            count += cursor.push(value)?;
                        }
                    });
                },
                syn::Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    let field_names_reversed = field_names.iter().rev();
                    
                    field_pop.push(quote! {
                        #i => Ok(#name::#ident {
                            #(#field_names_reversed: cursor.pop()?,)*
                        }),
                    });
                    field_push.push(quote! {
                        #name::#ident { #(#field_names),* } => {
                            count += cursor.push(#i as u8)?;
                            #(
                                count += cursor.push(#field_names)?;
                            )*
                        }
                    });
                }
            };
            (field_pop, field_push)
        },
    );

    let expanded = quote! {
        impl #impl_generics #squash_object for #name #ty_generics #where_clause {
            fn pop_obj<Obj>(cursor: &mut Obj) -> #result<Self>
            where
                Obj: #squash_cursor,
                Self: Sized {
                let tag = cursor.pop::<u8>()?;
                match tag {
                    #(#field_pop)*
                    _ => Err(::squash::Error::DeserializeVariantNotMatched),
                }
            }

            fn push_obj<Obj: #squash_cursor>(self, cursor: &mut Obj) -> #result<usize> {
                let mut count = 0;
                match self {
                    #(
                        #field_push
                    )*
                }
                Ok(count)
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
