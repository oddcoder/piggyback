extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Error, Expr, ItemFn, Local, Meta, Result, Stmt};

/// returns true if this is a piggy back attribute
fn is_piggyback_attr(attr: &Attribute) -> bool {
    let Meta::List(ref l) = attr.meta else {
        return false;
    };
    l.path.is_ident("piggyback")
}

/// extract the closure from the attribute
/// caller must verify that this is a piggyback attribute or else
/// they risk crashing
fn get_piggyback_closure(attr: Attribute) -> TokenStream2 {
    let Meta::List(l) = attr.meta else {
        unreachable!();
    };
    l.tokens
}

fn piggyback_actions_to_expr(
    expr: &Expr,
    closures: &[TokenStream2],
    is_async: bool,
) -> TokenStream2 {
    let mut base = if is_async {
        quote!(( || async {Ok(#expr)})().await)
    } else {
        quote!((|| {Ok(#expr)})())
    };
    for closure in closures {
        base = quote! (#base.map_err(|e| {(#closure)(&e); e}));
    }
    base
}

fn piggyback_local(mut local: Local, is_async: bool) -> Result<TokenStream2> {
    let span = local.span();
    let (pb_attrs, npb_attrs): (Vec<_>, Vec<_>) =
        local.attrs.into_iter().partition(is_piggyback_attr);
    local.attrs = npb_attrs;
    if pb_attrs.is_empty() {
        return Ok(quote! {#local});
    }
    let pb_actions: Vec<_> = pb_attrs.into_iter().map(get_piggyback_closure).collect();
    let Some(init) = local.init else {
        return Err(Error::new(span, "Variable declaration must be fully initialized. Please use `let x = value;` instead of `let x;`."));
    };
    if init.diverge.is_some() {
        unimplemented!();
    };
    let attrs = local.attrs;
    let pat = local.pat;
    let expr = piggyback_actions_to_expr(&init.expr, &pb_actions, is_async);
    Ok(quote! {
        #(#attrs)*
        let #pat = #expr?;
    })
}

fn piggyback_stmt(stmt: Stmt, is_async: bool) -> Result<TokenStream2> {
    match stmt {
        Stmt::Local(local) => piggyback_local(local, is_async),
        _ => Ok(quote! {#stmt}), //TODO
    }
}

fn piggyback_inner(item: ItemFn) -> Result<TokenStream2> {
    let is_async = item.sig.asyncness.is_some();
    let stmts: Vec<_> = item
        .block
        .stmts
        .into_iter()
        .map(|s| piggyback_stmt(s, is_async))
        .map(|x| x.unwrap())
        .collect();
    let attrs = item.attrs;
    let vis = item.vis;
    let sig = item.sig;
    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            #(#stmts)*
        }
    })
}
#[proc_macro_attribute]
pub fn piggyback(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    match piggyback_inner(item) {
        Err(e) => e.to_compile_error().into(),
        Ok(s) => s.into(),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_piggyback() {
        let t = trybuild::TestCases::new();
        t.pass("tests/pass/*.rs");
        t.compile_fail("tests/fail/*.rs")
    }
}
