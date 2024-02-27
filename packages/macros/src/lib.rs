use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput};

fn merge_variants(metadata: TokenStream, left: TokenStream, right: TokenStream) -> TokenStream {
    use syn::Data::Enum;

    // parse metadata
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    // parse the left enum
    let mut left: DeriveInput = parse_macro_input!(left);
    let Enum(DataEnum { variants, .. }) = &mut left.data else {
        return syn::Error::new(left.ident.span(), "only enums can accept variants")
            .to_compile_error()
            .into();
    };

    // parse the right enum
    let right: DeriveInput = parse_macro_input!(right);
    let Enum(DataEnum {
        variants: to_add, ..
    }) = right.data
    else {
        return syn::Error::new(left.ident.span(), "only enums can provide variants")
            .to_compile_error()
            .into();
    };

    // insert variants from the right to the left
    variants.extend(to_add.into_iter());

    quote! { #left }.into()
}

/// Note: `#[exchange_execute]` must be applied _before_ `#[cw_serde]`.
#[proc_macro_attribute]
pub fn exchange_execute(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                Swap {
                    minimum_receive_amount: ::cosmwasm_std::Coin,
                    route: Option<Binary>
                },
                SubmitOrder {
                    target_price: ::cosmwasm_std::Decimal256,
                    target_denom: String,
                },
                RetractOrder {
                    order_idx: ::cosmwasm_std::Uint128,
                    denoms: [String; 2],
                },
                WithdrawOrder {
                    order_idx: ::cosmwasm_std::Uint128,
                    denoms: [String; 2],
                },
                Receive(::cw20::Cw20ReceiveMsg),
            }
        }
        .into(),
    )
}

/// Note: `#[exchange_query]` must be applied _before_ `#[cw_serde]`.
#[proc_macro_attribute]
pub fn exchange_query(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                #[returns(Vec<Pair>)]
                GetPairs {
                    start_after: Option<::exchange::msg::Pair>,
                    limit: Option<u16>,
                },
                #[returns(::exchange::msg::Order)]
                GetOrder {
                    order_idx: ::cosmwasm_std::Uint128,
                    denoms: [String; 2],
                },
                #[returns(::cosmwasm_std::Decimal)]
                GetTwapToNow {
                    swap_denom: String,
                    target_denom: String,
                    period: u64,
                    route: Option<::cosmwasm_std::Binary>
                },
                #[returns(::cosmwasm_std::Coin)]
                GetExpectedReceiveAmount {
                    swap_amount: ::cosmwasm_std::Coin,
                    target_denom: String,
                    route: Option<::cosmwasm_std::Binary>
                }
            }
        }
        .into(),
    )
}
