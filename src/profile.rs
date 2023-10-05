use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, ExprLit, ExprPath, Token};

use crate::register::Register;
pub enum BaseAddr {
    Dyn,
    Static(syn::LitInt),
}
pub struct ProfileAttr {
    base_addr: BaseAddr,
    base_ty: syn::Path,
}

impl syn::parse::Parse for ProfileAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let exprs = Punctuated::<syn::Expr, Token![,]>::parse_separated_nonempty(input)?;
        let mut base_ty = None;
        let mut base_addr = BaseAddr::Static(syn::parse2::<syn::LitInt>(quote!(0)).unwrap());
        for expr in &exprs {
            if let syn::Expr::Path(ExprPath { path, .. }) = expr {
                if path.is_ident("u8")
                    || path.is_ident("u16")
                    || path.is_ident("u32")
                    || path.is_ident("u64")
                    || path.is_ident("u128")
                {
                    base_ty = Some(path.to_owned());
                }
                if path.is_ident("DynBase") {
                    base_addr = BaseAddr::Dyn;
                }
            }
            if let syn::Expr::Lit(ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) = expr
            {
                base_addr = BaseAddr::Static(lit_int.to_owned());
            }
        }
        Ok(ProfileAttr {
            base_addr,
            base_ty: base_ty.ok_or(syn::Error::new(
                exprs.span(),
                "Register's base type must be specified(u8, u16, u32, u64, u128)",
            ))?,
        })
    }
}
pub struct ProfileItem {
    pub name: syn::Ident,
    pub vis: syn::Visibility,
    pub doc: Vec<syn::Attribute>,
    pub registers: Vec<Register>,
}

impl TryFrom<syn::ItemStruct> for ProfileItem {
    type Error = syn::Error;

    fn try_from(st: syn::ItemStruct) -> Result<Self, Self::Error> {
        if st.generics.lt_token.is_some() {
            return Err(syn::Error::new_spanned(
                st.generics,
                "Generics for register profile is not supported",
            ));
        }
        let mut doc = Vec::new();
        for attr in &st.attrs {
            if attr.path().is_ident("doc") {
                doc.push(attr.to_owned());
            }
        }
        let name = st.ident.to_owned();
        let vis = st.vis.to_owned();
        let mut registers = Vec::new();
        for field in &st.fields {
            let register: Register = field.try_into()?;
            registers.push(register);
        }
        Ok(ProfileItem {
            name,
            vis,
            doc,
            registers,
        })
    }
}

impl syn::parse::Parse for ProfileItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_st = syn::ItemStruct::parse(input)?;
        item_st.try_into()
    }
}

pub struct Profile {
    pub attr: ProfileAttr,
    pub item: ProfileItem,
}

impl ToTokens for Profile {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let profile_name = &self.item.name;
        let profile_vis = &self.item.vis;
        let base_ty = &self.attr.base_ty;
        let mut base_addr = quote!();
        if let BaseAddr::Dyn = self.attr.base_addr {
            tokens.extend(quote! {
                #profile_vis struct #profile_name(#profile_vis usize);
            });
            base_addr.extend(quote!(#profile_name.0));
        } else if let BaseAddr::Static(base_lit) = &self.attr.base_addr {
            tokens.extend(quote! {
                #profile_vis struct #profile_name;
            });
            base_addr.extend(quote!(#base_lit));
        }
        for register in &self.item.registers {
            let offset = &register.offset;
            let name = &register.name;
            let cache_ty = &register.cache_ty;
            tokens.extend(quote! {
                #register
                impl ::register::Register for #name {
                    type CacheType = #cache_ty;
                }
                impl ::register::Profile<#name> for #profile_name {
                    unsafe fn cache(reg: #name) -> <#name as ::register::Register>::CacheType {
                        let addr = #base_addr + #offset;
                        let raw = ::core::ptr::read_volatile(addr as *const #base_ty);
                        #cache_ty(raw)
                    }
                    unsafe fn flush(reg: #name, cache: <#name as ::register::Register>::CacheType) {
                        let raw = cache.0;
                        let addr = #base_addr + #offset;
                        ::core::ptr::write_volatile(addr as *mut #base_ty, raw);
                    }
                }
            });
        }
    }
}
