use crate::{msg::ExecuteMsg, types::destination::Destination};
use base::vaults::vault::{OldDestination, PostExecutionAction};
use cosmwasm_std::{to_binary, Addr};

pub fn destination_from(
    old_destination: &OldDestination,
    owner: Addr,
    old_staking_router_address: Addr,
) -> Destination {
    match old_destination.action.clone() {
        PostExecutionAction::Send => Destination {
            address: old_destination.address.clone(),
            allocation: old_destination.allocation,
            msg: None,
        },
        PostExecutionAction::ZDelegate => Destination {
            address: old_staking_router_address,
            allocation: old_destination.allocation,
            msg: Some(
                to_binary(&ExecuteMsg::ZDelegate {
                    delegator_address: owner,
                    validator_address: old_destination.address.clone(),
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
    use cosmwasm_std::{to_binary, Addr, Decimal};

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
            Addr::unchecked("staking-router"),
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
            Addr::unchecked("staking-router"),
        );

        assert_eq!(
            destination,
            Destination {
                allocation: old_destination.allocation,
                address: Addr::unchecked("staking-router"),
                msg: Some(
                    to_binary(&ExecuteMsg::ZDelegate {
                        delegator_address: Addr::unchecked("owner"),
                        validator_address: old_destination.address
                    })
                    .unwrap()
                )
            }
        );
    }
}
