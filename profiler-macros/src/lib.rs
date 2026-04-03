use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprCall, ExprMacro, ExprMethodCall, ExprPath, Ident, LitStr};

// ─── Name extraction ─────────────────────────────────────────────────────────

fn derive_name(expr: &Expr) -> String {
    match expr {
        Expr::Call(ExprCall { func, .. }) => match func.as_ref() {
            Expr::Path(ExprPath { path, .. }) => path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_else(|| fallback_name(expr)),
            other => fallback_name(other),
        },
        Expr::MethodCall(ExprMethodCall { method, .. }) => method.to_string(),
        Expr::Macro(ExprMacro { mac, .. }) => mac
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_else(|| fallback_name(expr)),
        _ => fallback_name(expr),
    }
}

fn fallback_name(expr: &Expr) -> String {
    let s = quote!(#expr).to_string();
    if s.len() > 32 {
        format!("{}...", &s[..29])
    } else {
        s
    }
}

// ─── profile!(expr) ──────────────────────────────────────────────────────────

/// Wraps an expression, recording elapsed time and call count.
///
/// ```rust
/// let result = profile!(compute_fib(40));
/// ```
#[proc_macro]
pub fn timed(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    let name = derive_name(&expr);
    let name_lit = LitStr::new(&name, Span::call_site());

    // Hygienic temporaries: mixed_site keeps them invisible to the caller's scope
    let t0 = Ident::new("__profiler_t0", Span::mixed_site());
    let result = Ident::new("__profiler_result", Span::mixed_site());

    let expanded = quote! {
        {
            let #t0 = ::std::time::Instant::now();
            let #result = #expr;
            ::profiler::record(#name_lit, #t0.elapsed().as_nanos() as u64);
            #result
        }
    };

    TokenStream::from(expanded)
}

// ─── summarise!() ────────────────────────────────────────────────────────────

/// Prints a formatted summary table of all profiling data for the current thread.
///
/// ```rust
/// summarise!();
/// ```
#[proc_macro]
pub fn summarise(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "`summarise!()` takes no arguments")
            .to_compile_error()
            .into();
    }

    TokenStream::from(quote! { ::profiler::print_summary() })
}

// ─── profile_count!(name) ────────────────────────────────────────────────────

/// Increments a named counter (no timing).
/// Accepts a string literal or a bare identifier.
///
/// ```rust
/// profile_count!("cache_hit");
/// profile_count!(cache_hit);  // equivalent
/// ```
#[proc_macro]
pub fn count(input: TokenStream) -> TokenStream {
    let name: String = if let Ok(lit) = syn::parse::<LitStr>(input.clone()) {
        lit.value()
    } else if let Ok(ident) = syn::parse::<Ident>(input) {
        ident.to_string()
    } else {
        return syn::Error::new(
            Span::call_site(),
            "`profile_count!` expects a string literal or a bare identifier",
        )
        .to_compile_error()
        .into();
    };

    let name_lit = LitStr::new(&name, Span::call_site());
    TokenStream::from(quote! { ::profiler::increment(#name_lit) })
}
