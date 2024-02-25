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

        for (i, field) in fields.iter().enumerate() {
            let name = match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None        => syn::Member::Unnamed(i.into()),
            };

            result.extend(quote! {
                let offset = (&self.#name as *const _ as usize) - (self as *const _ as usize);
                bytes.para_slice_mut(prev_end..offset).fill(0);

                let size = ::core::mem::size_of_val(&self.#name);
                let prev_end = offset + size;
                self.#name.clear_padding(bytes.para_slice_mut(offset..prev_end));
            });
        }

        result.extend(quote! {
            let offset = ::core::mem::size_of::<Self>();
            bytes.para_slice_mut(prev_end..offset).fill(0);
        });

        result
    };


    let result = quote! {
        unsafe impl #impl_generics ::wenjin::CType for #ident #ty_generics #where_clause {
            #[inline(always)]
            unsafe fn clear_padding(&self, mut bytes: ::wenjin::ParaSliceMut<u8>) {
                unsafe { bytes.assume_len(::core::mem::size_of::<Self>()) }

                #clear_padding
            }
        }
    };

    TokenStream::from(result)
}

