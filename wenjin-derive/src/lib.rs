extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};


// @todo: does this actually need to be in a separate crate?


#[proc_macro_derive(CType)]
pub fn derive_ctype(item: TokenStream) -> TokenStream {
    let DeriveInput { attrs, vis: _, ident, generics, data } = parse_macro_input!(item as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();


    // ensure is struct.
    let syn::Data::Struct(data) = data else {
        panic!("CType: item is not a struct.");
    };
    let fields = &data.fields;


    // ensure is `repr(C)`.
    let mut is_repr_c = false;
    for attr in &attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    is_repr_c = true;
                }
                Ok(())
            }).unwrap();
        }
    }
    assert!(is_repr_c, "CType: struct must be `repr(C)`");


    // generate code to clear the padding.
    // this also ensures all fields implement `CType`.
    let clear_padding = {
        let mut result = proc_macro2::TokenStream::new();

        result.extend(quote! {
            let prev_end = 0;
        });

        for field in fields.iter() {
            let ty = &field.ty;
            result.extend(quote! {
                let offset = ::wenjin::ceil_to_multiple_pow2(prev_end, ::core::mem::align_of::<#ty>());
                unsafe { bytes.get_mut(prev_end..offset).unwrap_unchecked().fill(0) };

                let size = ::core::mem::size_of::<#ty>();
                let prev_end = offset + size;
                <#ty as ::wenjin::CType>::clear_padding(unsafe { bytes.get_mut(offset..prev_end).unwrap_unchecked() });
            });
        }

        result.extend(quote! {
            let offset = ::wenjin::ceil_to_multiple_pow2(prev_end, ::core::mem::align_of::<Self>());
            debug_assert_eq!(offset, ::core::mem::size_of::<Self>());
            unsafe { bytes.get_mut(prev_end..offset).unwrap_unchecked().fill(0) };
        });

        result
    };


    let result = quote! {
        unsafe impl #impl_generics ::wenjin::CType for #ident #ty_generics #where_clause {
            #[inline(always)]
            unsafe fn clear_padding(bytes: &mut [u8]) {
                debug_assert_eq!(bytes.len(), ::core::mem::size_of::<Self>());
                #clear_padding
            }
        }
    };

    TokenStream::from(result)
}

