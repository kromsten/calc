use crate::types::destination::Destination;
use base::vaults::vault::{OldDestination, PostExecutionAction};
use cosmwasm_std::Addr;

pub fn destination_from(
    old_destination: &OldDestination,
    _owner: Addr,
    _contract_address: Addr,
) -> Destination {
    match old_destination.action.clone() {
        PostExecutionAction::Send => Destination {
            address: old_destination.address.clone(),
            allocation: old_destination.allocation,
            msg: None,
        },
        PostExecutionAction::ZDelegate => panic!("ZDelegate is not supported yet"),
        // PostExecutionAction::ZDelegate => Destination {
        //     address: contract_address,
        //     allocation: old_destination.allocation,
        //     msg: Some(
        //         to_binary(&ExecuteMsg::ZDelegate {
        //             delegator_address: owner,
        //             validator_address: old_destination.address.clone(),
        //         })
        //         .unwrap(),
        //     ),
        // },
        // PostExecutionAction::Custom { contract_addr, msg } => Destination {
        //     address: contract_addr,
        //     allocation: old_destination.allocation,
        //     msg,
        // },
    }
}
