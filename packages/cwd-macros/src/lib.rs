use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput, Variant};

/// Adds the necessary fields to an such that the enum implements the
/// interface needed to be a voting module.
///
/// For example:
///
/// ```
/// use cwd_macros::voting_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
///
/// #[voting_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     VotingPowerAtHeight {
///       address: String,
///       height: Option<u64>
///     },
///     TotalPowerAtHeight {
///       height: Option<u64>
///     },
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::voting_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::{TotalPowerAtHeightResponse, VotingPowerAtHeightResponse};
/// use cosmwasm_std::Empty;
///
/// #[derive(Clone)]
/// #[voting_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn voting_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "voting query macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let voting_power: Variant = syn::parse2(quote! {
                #[returns(VotingPowerAtHeightResponse)]
                VotingPowerAtHeight {
                    address: ::std::string::String,
                    height: ::std::option::Option<::std::primitive::u64>
                }
            })
            .unwrap();

            let total_power: Variant = syn::parse2(quote! {
                #[returns(TotalPowerAtHeightResponse)]
                TotalPowerAtHeight {
                    height: ::std::option::Option<::std::primitive::u64>
                }
            })
            .unwrap();

            // This is example how possible we can implement such methods,
            // but there is requirement to modify core contract (so for now it makes nno sense)
            // let claims: Variant = syn::parse2(quote! { Claims {
            //     address: ::std::string::String,
            // } })
            //     .unwrap();
            //
            // let list_stakers: Variant = syn::parse2(quote! { ListStakers {
            //     address: ::std::option::Option<::std::string::String>,
            //     height: ::std::option::Option<::std::primitive::u32>
            // } })
            //     .unwrap();

            variants.push(voting_power);
            variants.push(total_power);
            // variants.push(claims);
            // variants.push(list_stakers);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "voting query types can not be only be derived for enums",
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

/// Adds the necessary fields to an enum such that it implements the
/// interface needed to be a voting module with a token.
///
/// For example:
///
/// ```
/// use cwd_macros::token_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Addr;
///
/// #[token_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     TokenContract {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::token_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::{Empty, Addr};
///
/// #[derive(Clone)]
/// #[token_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn token_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "token query macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let info: Variant = syn::parse2(quote! {
                #[returns(Addr)]
                TokenContract {}
            })
            .unwrap();

            variants.push(info);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "token query types can not be only be derived for enums",
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

/// Adds the necessary fields to an enum such that it implements the
/// interface needed to be a voting module that has an
/// active check threshold.
///
/// For example:
///
/// ```
/// use cwd_macros::active_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Empty;
/// use cwd_interface::voting::IsActiveResponse;
///
/// #[active_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     IsActive {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::active_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Empty;
/// use cwd_interface::voting::IsActiveResponse;
///
/// #[derive(Clone)]
/// #[active_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn active_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "token query macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let info: Variant = syn::parse2(quote! {
                #[returns(IsActiveResponse)]
                IsActive {}
            })
            .unwrap();

            variants.push(info);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "token query types can not be only be derived for enums",
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

/// Adds the necessary fields to an enum such that it implements the
/// interface needed to be a module that has an
/// info query.
///
/// For example:
///
/// ```
/// use cwd_macros::info_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::InfoResponse;
///
/// #[info_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     Info {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::info_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::InfoResponse;
/// use cosmwasm_std::Empty;
///
/// #[info_query]
/// #[derive(Clone)]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn info_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "info query macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let info: Variant = syn::parse2(quote! {
                #[returns(InfoResponse)]
                Info {}
            })
            .unwrap();

            variants.push(info);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "info query types can not be only be derived for enums",
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

/// Adds the necessary fields to an enum such that it implements the
/// interface needed to be a proposal module.
///
/// For example:
///
/// ```
/// use cwd_macros::proposal_module_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Addr;
///
/// #[proposal_module_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     Dao {},
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::proposal_module_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::{Addr, Empty};
///
/// #[derive(Clone)]
/// #[proposal_module_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn proposal_module_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    // Make sure that no arguments were passed in.
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "govmod query macro takes no arguments")
            .to_compile_error()
            .into();
    }

    let mut ast: DeriveInput = parse_macro_input!(input);
    match &mut ast.data {
        syn::Data::Enum(DataEnum { variants, .. }) => {
            let dao: Variant = syn::parse2(quote! {
                #[returns(Addr)]
                Dao {}
            })
            .unwrap();

            variants.push(dao);
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "govmod query types can not be only be derived for enums",
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

/// Limits the number of variants allowed on an enum at compile
/// time. For example, the following will not compile:
///
/// ```compile_fail
/// use cwd_macros::limit_variant_count;
///
/// #[limit_variant_count(1)]
/// enum Two {
///     One {},
///     Two {},
/// }
/// ```
#[proc_macro_attribute]
pub fn limit_variant_count(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(metadata as AttributeArgs);
    if args.len() != 1 {
        panic!("macro takes one argument. ex: `#[limit_variant_count(4)]`")
    }

    let limit: usize = if let syn::NestedMeta::Lit(syn::Lit::Int(unparsed)) = args.first().unwrap()
    {
        match unparsed.base10_parse() {
            Ok(limit) => limit,
            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        return syn::Error::new_spanned(args[0].clone(), "argument should be an integer literal")
            .to_compile_error()
            .into();
    };

    let ast: DeriveInput = parse_macro_input!(input);
    match ast.data {
        syn::Data::Enum(DataEnum { ref variants, .. }) => {
            if variants.len() > limit {
                return syn::Error::new_spanned(
                    variants[limit].clone(),
                    format!("this enum's variant count is limited to {}", limit),
                )
                .to_compile_error()
                .into();
            }
        }
        _ => {
            return syn::Error::new(
                ast.ident.span(),
                "limit_variant_count may only be derived for enums",
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

/// Adds the necessary fields to an enum such that the enum implements the
/// query interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use cwd_macros::pausable_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
///
/// #[cw_serde]
/// struct PauseInfoResponse{}
///
/// #[pausable_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     /// Returns information about if the contract is currently paused.
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
/// use cwd_macros::pausable_query;
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cosmwasm_std::Empty;
///
/// struct PauseInfoResponse{}
///
/// #[derive(Clone)]
/// #[pausable_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn pausable_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
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
            let pause_info: Variant = syn::parse2(quote! {
                #[returns(PauseInfoResponse)]
                PauseInfo {}
            })
            .unwrap();

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

/// Adds the necessary fields to an enum such that the enum implements the
/// interface needed to be paused/unpaused.
///
/// For example:
///
/// ```
/// use cwd_macros::pausable;
///
/// #[pausable]
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
/// use cwd_macros::pausable;
///
/// #[derive(Clone)]
/// #[pausable]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn pausable(metadata: TokenStream, input: TokenStream) -> TokenStream {
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

/// Adds the necessary fields to an enum such that the enum implements the
/// voting vault execution interface.
///
/// For example:
///
/// ```
/// use cwd_macros::voting_vault;
///
/// #[voting_vault]
/// enum ExecuteMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum ExecuteMsg {
//      Bond {},
//      Unbond {
//          amount: Uint128,
//      },
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::voting_vault;
///
/// #[derive(Clone)]
/// #[voting_vault]
/// #[allow(dead_code)]
/// enum Test {
///     Foo,
///     Bar(u64),
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn voting_vault(metadata: TokenStream, input: TokenStream) -> TokenStream {
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
            let bond: Variant = syn::parse2(quote! { Bond {} }).unwrap();
            let unbond: Variant = syn::parse2(quote! { Unbond {
                amount: ::cosmwasm_std::Uint128
            } })
            .unwrap();

            variants.push(bond);
            variants.push(unbond);
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

/// Adds the necessary fields to an enum such that the enum implements the
/// voting vault query interface.
///
/// For example:
///
/// ```
/// use cwd_macros::voting_vault_query;
/// use cosmwasm_std::{Uint128, Addr};
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::BondingStatusResponse;
///
/// #[voting_vault_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// enum QueryMsg {}
/// ```
///
/// Will transform the enum to:
///
/// ```
/// enum QueryMsg {
///     /// Returns bonding status for the given address. The bonding status tells whether the
///     /// vault is open for bonding and whether the address is eligible for unbonding funds.
///     BondingStatus {
///         address: String,
///         height: Option<u64>
///     },
///     /// Returns the address of the DAO behind the vault.
///     Dao {},
///     /// The name of the vault to ease recognition.
///     Name {},
///     /// Returns the vault's description.
///     Description {},
///     /// Returns the list of addresses bonded to this vault and along with the bonded balances.
///     ListBonders {
///         start_after: Option<String>,
///         limit: Option<u32>,
///     },
/// }
/// ```
///
/// Note that other derive macro invocations must occur after this
/// procedural macro as they may depend on the new fields. For
/// example, the following will fail becase the `Clone` derivation
/// occurs before the addition of the field.
///
/// ```compile_fail
/// use cwd_macros::voting_vault_query;
/// use cosmwasm_std::{Uint128, Addr, Empty};
/// use cosmwasm_schema::{cw_serde, QueryResponses};
/// use cwd_interface::voting::BondingStatusResponse;
///
/// #[derive(Clone)]
/// #[voting_vault_query]
/// #[cw_serde]
/// #[derive(QueryResponses)]
/// #[allow(dead_code)]
/// enum Test {
///     #[returns(Empty)]
///     Foo,
///     #[returns(Empty)]
///     Bar(u64),
///     #[returns(Empty)]
///     Baz { foo: u64 },
/// }
/// ```
#[proc_macro_attribute]
pub fn voting_vault_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
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
            let bonding_status: Variant = syn::parse2(quote! {
                   #[returns(BondingStatusResponse)]
                   BondingStatus {
                       address: ::std::string::String,
                       height: ::std::option::Option<::std::primitive::u64>
                   }
            })
            .unwrap();
            let dao: Variant = syn::parse2(quote! {
                #[returns(Addr)]
                Dao {}
            })
            .unwrap();
            let name: Variant = syn::parse2(quote! {
                #[returns(String)]
                Name {}
            })
            .unwrap();
            let description: Variant = syn::parse2(quote! {
                #[returns(String)]
                Description {}
            })
            .unwrap();
            let bonders: Variant = syn::parse2(quote! {
                #[returns(Vec<(Addr, Uint128)>)]
                ListBonders {
                    start_after: ::std::option::Option<::std::string::String>,
                    limit: ::std::option::Option<::std::primitive::u32>
                }
            })
            .unwrap();

            variants.push(bonding_status);
            variants.push(dao);
            variants.push(name);
            variants.push(description);
            variants.push(bonders);
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
