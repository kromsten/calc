use crate::{msg::ExecuteMsg, types::destination::Destination};
use base::vaults::vault::{OldDestination, PostExecutionAction};
use cosmwasm_std::{to_binary, Addr, Uint128};

pub fn destination_from(
    old_destination: &OldDestination,
    owner: Addr,
    contract_address: Addr,
) -> Destination {
    match old_destination.action.clone() {
        PostExecutionAction::Send => Destination {
            address: old_destination.address.clone(),
            allocation: old_destination.allocation,
            msg: None,
        },
        PostExecutionAction::ZDelegate => Destination {
            address: contract_address,
            allocation: old_destination.allocation,
            msg: Some(
                to_binary(&ExecuteMsg::OldZDelegate {
                    delegator_address: owner,
                    validator_address: old_destination.address.clone(),
                    amount: Uint128::zero(),
                    denom: "".to_string(),
                })
                .unwrap(),
            ),
        },
    }
}

#[cfg(test)]
mod destination_from_tests {
    use super::destination_from;
    use crate::{msg::ExecuteMsg, types::destination::Destination};
    use base::vaults::vault::{OldDestination, PostExecutionAction};
    use cosmwasm_std::{to_binary, Addr, Decimal, Uint128};

    #[test]
    fn maps_send_destination_correctly() {
        let old_destination = OldDestination {
            address: Addr::unchecked("user"),
            allocation: Decimal::percent(15),
            action: PostExecutionAction::Send,
        };

        let destination = destination_from(
            &old_destination,
            Addr::unchecked("owner"),
            Addr::unchecked("contract"),
        );

        assert_eq!(
            destination,
            Destination {
                allocation: old_destination.allocation,
                address: old_destination.address,
                msg: None
            }
        )
    }

    #[test]
    fn maps_zdelegate_destination_correctly() {
        let old_destination = OldDestination {
            address: Addr::unchecked("user"),
            allocation: Decimal::percent(15),
            action: PostExecutionAction::ZDelegate,
        };

        let destination = destination_from(
            &old_destination,
            Addr::unchecked("owner"),
            Addr::unchecked("contract"),
        );

        assert_eq!(
            destination,
            Destination {
                allocation: old_destination.allocation,
                address: Addr::unchecked("contract"),
                msg: Some(
                    to_binary(&ExecuteMsg::OldZDelegate {
                        delegator_address: Addr::unchecked("owner"),
                        validator_address: old_destination.address,
                        amount: Uint128::zero(),
                        denom: "".to_string()
                    })
                    .unwrap()
                )
            }
        );
    }
}
