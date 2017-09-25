extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

fn get_attr(attrs: &[syn::Attribute], name: &str) -> String {
    let attr = attrs.iter().find(| &attr | attr.name() == name)
        .expect(format!("Requires '{}' attribute", name).as_str());

    let lit = match &attr.value {
        &syn::MetaItem::NameValue(_, ref lit) => lit,
        _ => panic!("Invalid attribute meta item")
    };

    match lit {
        &syn::Lit::Str(ref val, _) => val.clone(),
        _ => panic!("Invalid attribute value type")
    }
}

#[proc_macro_derive(Handle, attributes(HandleType, HandleTrait, HandleRaw))]
pub fn handle(input: TokenStream) -> TokenStream {
    let source = input.to_string();

    // Parse a string representation as an AST
    let ast = syn::parse_derive_input(&source).unwrap();

    let gen = impl_handle(&ast);
    gen.parse().unwrap()
}

fn impl_handle(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;

    let ty  = syn::Ident::new(get_attr(&ast.attrs, "HandleType"));
    let tr  = syn::Ident::new(get_attr(&ast.attrs, "HandleTrait"));
    let raw = syn::Ident::new(get_attr(&ast.attrs, "HandleRaw"));

    quote! {
        impl #tr for #ident {
            fn from_raw(raw: #raw) -> Self {
                #ident(raw)
            }

            fn as_raw(&self) -> #raw {
                self.0
            }
        }

        impl ::std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}::Handle({})", "#ty", self.0)
            }
        }
    }
}
