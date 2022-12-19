use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the necessary fields to an enum such that the enum implements the command
/// interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use exec_control_macro::exec_controlled;
///
/// #[exec_controlled]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum ExecuteMsg {
///     Pause { duration: u64 },
///     Unpause {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use exec_control_macro::exec_controlled;
///
/// #[derive(Clone)]
/// #[exec_controlled]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn exec_controlled(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "pausing cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let pause: Variant = syn::parse2(quote! { Pause {
                duration: ::std::primitive::u64
            } })
            .unwrap();
            let unpause: Variant = syn::parse2(quote! { Unpause {} }).unwrap();

            variants.push(pause);
            variants.push(unpause);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "pausing cmd types can not be only be derived for enums",
            )
            .to_compile_error()
            .into()
        }
    };

    quote! {
    #ast
    }
    .into()
}

/// Adds the necessary fields to an enum such that the enum implements the query
/// interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use exec_control_macro::exec_controlled_query;
///
/// #[exec_controlled_query]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     PauseInfo {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use exec_control_macro::exec_controlled_query;
///
/// #[derive(Clone)]
/// #[exec_controlled_query]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn exec_controlled_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "pausing cmd macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let pause_info: Variant = syn::parse2(quote! { PauseInfo {} }).unwrap();

            variants.push(pause_info);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "pausing cmd types can not be only be derived for enums",
            )
            .to_compile_error()
            .into()
        }
    };

    quote! {
    #ast
    }
    .into()
}
