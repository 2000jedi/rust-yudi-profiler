use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Expr, Ident, ItemFn, LitStr, parse_macro_input};

// ─── #[timed] ────────────────────────────────────────────────────────────────

/// Instruments a function, recording elapsed time and call count for each call.
///
/// ```ignore
/// #[timed]
/// fn compute_fib(n: u64) -> u64 {
///     // ...
///     n
/// }
/// ```
#[proc_macro_attribute]
pub fn timed(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(Span::call_site(), "`#[timed]` takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut function = parse_macro_input!(item as ItemFn);
    let name = function.sig.ident.to_string();
    let name_lit = LitStr::new(&name, Span::call_site());

    // A guard records timing even when the function exits early or panics.
    let guard = Ident::new("_profiler_guard", Span::mixed_site());
    let block = function.block;
    function.block = Box::new(syn::parse_quote! {
        {
            let #guard = ::profiler::TimingGuard::new(#name_lit);
            #block
        }
    });

    TokenStream::from(quote!(#function))
}

// ─── summarise!() ────────────────────────────────────────────────────────────

/// Prints a formatted summary table of all profiling data for the current thread.
///
/// ```ignore
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

// ─── count!(name) ─────────────────────────────────────────────────────────────

/// Increments a named counter (no timing).
/// Accepts a string literal or a bare identifier.
///
/// ```ignore
/// count!("cache_hit");
/// count!(cache_hit);  // equivalent
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
            "`count!` expects a string literal or a bare identifier",
        )
        .to_compile_error()
        .into();
    };

    let name_lit = LitStr::new(&name, Span::call_site());
    TokenStream::from(quote! { ::profiler::increment(#name_lit) })
}

// ─── summarise_csv!() ────────────────────────────────────────────────────────

/// Prints the current thread's profile data as CSV (header + rows) to stdout.
///
/// ```ignore
/// summarise_csv!();
/// ```
#[proc_macro]
pub fn summarise_csv(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        return syn::Error::new(Span::call_site(), "`summarise_csv!()` takes no arguments")
            .to_compile_error()
            .into();
    }
    TokenStream::from(quote! { ::profiler::print_csv() })
}

// ─── append_file!(target) ────────────────────────────────────────────────────

/// Appends the current thread's profile data as CSV rows to a target.
/// Accepts any value implementing `profiler::AppendTarget` — string literals,
/// `&str` / `String`, `&Path` / `PathBuf`, `&File` / `&mut File`, etc.
/// Writes a header row if the target file is missing or empty.
/// Returns `std::io::Result<()>`.
///
/// ```ignore
/// append_file!("profile.csv").unwrap();
/// let path: PathBuf = "profile.csv".into();
/// append_file!(&path).unwrap();
/// let mut f = std::fs::OpenOptions::new().create(true).append(true).open("p.csv")?;
/// append_file!(&mut f).unwrap();
/// ```
#[proc_macro]
pub fn append_file(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    TokenStream::from(quote! { ::profiler::AppendTarget::append_profile(#expr) })
}
