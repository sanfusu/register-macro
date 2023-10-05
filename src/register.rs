use quote::{quote, ToTokens};

pub struct Register {
    pub offset: syn::LitInt,
    pub doc: Vec<syn::Attribute>,
    pub name: syn::Ident,
    pub cache_ty: syn::Type,
    pub vis: syn::Visibility,
}

impl TryFrom<&syn::Field> for Register {
    type Error = syn::Error;

    fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
        let mut offset = None;
        let mut doc = Vec::new();
        for attr in &field.attrs {
            if attr.path().is_ident("offset") {
                offset = Some(attr.parse_args::<syn::LitInt>()?);
            }
            if attr.path().is_ident("doc") {
                doc.push(attr.to_owned());
            }
        }
        let name = field.ident.to_owned().ok_or(syn::Error::new_spanned(
            field,
            "Register's name must be specified",
        ))?;
        let cache_ty = field.ty.to_owned();
        let vis = field.vis.to_owned();
        Ok(Register {
            offset: offset.ok_or(syn::Error::new_spanned(
                field,
                "offset for register must be specified",
            ))?,
            doc,
            name,
            cache_ty,
            vis,
        })
    }
}

impl ToTokens for Register {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let doc = &self.doc;
        let name = &self.name;
        let vis = &self.vis;
        tokens.extend(quote! {
            #(#doc)*
            #vis struct #name;
        })
    }
}
