use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CosmosMsg, Deps, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use neutron_sdk::bindings::msg::{
    AdminProposal, NeutronMsg, ParamChangeProposal, ProposalExecuteMessage,
};

use std::collections::HashSet;
use std::iter::FromIterator;

use crate::contract::get_cron_params;
use crate::cron_module_param_types::{MsgUpdateParamsCron, ParamsRequestCron};
use crate::msg::{CronUpdateParamsPermission, ParamChangePermission, ParamPermission};
use crate::{
    cron_module_param_types::MSG_TYPE_UPDATE_PARAMS_CRON, error::ContractError,
    msg::ProposalExecuteMessageJSON,
};

#[cw_serde]
pub enum Permission {
    AllowAll,
    AddCronPermission,
    RemoveCronPermission,
    CronUpdateParamsPermission(CronUpdateParamsPermission),
    ParamChangePermission(ParamChangePermission),
}

fn remove_length_prefix(value: Vec<u8>) -> Vec<u8> {
    // That is required due to `Prefixer` and `PrimaryKey` implementations for PermissionType
    // ```
    //  serde_json_wasm::to_vec(self).unwrap().iter().map(|&v| Key::Val8([v])).collect()
    // ```
    // The thing is, during deriving storage key, every element of the Vec<Kev::Val8> 
    // except the last one becomes prefixed by a length of a key, for example "aaa" (bytes 65 65 65) 
    // converts into 0 1 65 + 0 1 65 + 65, where 0 1 is 2bytes length
    // we converting back prefixed vec to non prefixed
    let len_prefix_length = 2;
    let mut v = value
        .iter()
        .skip(len_prefix_length)
        .step_by(len_prefix_length + 1)
        .collect::<Vec<&u8>>();
    v.push(&value[value.len() - 1]);
    v.into_iter().map(|&v| v).collect::<Vec<u8>>()
}

impl<'a> KeyDeserialize for PermissionType {
    type Output = PermissionType;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        serde_json_wasm::from_slice(remove_length_prefix(value).as_ref())
            .map_err(|e| StdError::generic_err(format!("Invalid PermissionType: {}", e)))
    }
}

impl<'a> PrimaryKey<'a> for PermissionType {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // TODO: find a way use Key::Ref insated of Key::Val8
        // to simplify KeyDeserialize and remove `remove_length_prefix`
        serde_json_wasm::to_vec(self).unwrap().iter().map(|&v| Key::Val8([v])).collect()
    }
}

impl<'a> Prefixer<'a> for PermissionType {
    fn prefix(&self) -> Vec<Key> {
        serde_json_wasm::to_vec(self).unwrap().iter().map(|&v| Key::Val8([v])).collect()
    }
}


#[cw_serde]
pub enum PermissionType {
    AllowAll,
    AddCronPermission,
    RemoveCronPermission,
    CronUpdateParamsPermission,
    ParamChangePermission,
}

pub enum Validator {
    Empty,
    CronUpdateParams(MsgUpdateParamsCron),
    ParamChange(ParamChangeProposal),
}

impl Validator {
    pub fn validate(self, deps: Deps, permission: Permission) -> Result<(), ContractError> {
        // match only possible variants
        match (self, permission) {
            (Validator::Empty, Permission::AllowAll) => Ok(()),
            (Validator::Empty, Permission::AddCronPermission) => Ok(()),
            (Validator::Empty, Permission::RemoveCronPermission) => Ok(()),
            (
                Validator::CronUpdateParams(msg),
                Permission::CronUpdateParamsPermission(permissions),
            ) => check_cron_update_msg_params(deps, permissions, msg),
            (
                Validator::ParamChange(params_change_proposal),
                Permission::ParamChangePermission(permissions),
            ) => check_param_change_permission(params_change_proposal, permissions),
            _ => unreachable!(),
        }
    }
}

// match_permission_type returns PermissionType and Validator, for futher processing
// Validator contains message payload if necessary
pub fn match_permission_type(
    msg: &CosmosMsg<NeutronMsg>,
) -> Result<(PermissionType, Validator), ContractError> {
    match msg {
        CosmosMsg::Custom(NeutronMsg::AddSchedule { .. }) => {
            Ok((PermissionType::AddCronPermission, Validator::Empty))
        }
        CosmosMsg::Custom(NeutronMsg::RemoveSchedule { .. }) => {
            Ok((PermissionType::RemoveCronPermission, Validator::Empty))
        }
        CosmosMsg::Custom(NeutronMsg::SubmitAdminProposal { admin_proposal }) => {
            match admin_proposal {
                AdminProposal::ProposalExecuteMessage(ProposalExecuteMessage { message }) => {
                    let typed_proposal: ProposalExecuteMessageJSON =
                        serde_json_wasm::from_str(message.as_str())?;

                    if typed_proposal.type_field.as_str() == MSG_TYPE_UPDATE_PARAMS_CRON {
                        let msg_update_params: MsgUpdateParamsCron =
                            serde_json_wasm::from_str(message.as_str())?;
                        return Ok((
                            PermissionType::CronUpdateParamsPermission,
                            Validator::CronUpdateParams(msg_update_params),
                        ));
                    };
                    Err(ContractError::PermissionTypeNotFound(
                        "no registered admin proposal persimmisions for the message".to_string(),
                    ))
                }
                AdminProposal::ParamChangeProposal(param_change_proposal) => Ok((
                    PermissionType::ParamChangePermission,
                    Validator::ParamChange(param_change_proposal.to_owned()),
                )),
                _ => Err(ContractError::PermissionTypeNotFound(
                    "no registered admin proposal persimmisions for the message".to_string(),
                )),
            }
        }
        _ => Err(ContractError::PermissionTypeNotFound(
            "no registered persimmisions for the message".to_string(),
        )),
    }
}

impl Into<PermissionType> for Permission {
    fn into(self) -> PermissionType {
        match self {
            Permission::AddCronPermission => PermissionType::AddCronPermission,
            Permission::RemoveCronPermission => PermissionType::RemoveCronPermission,
            Permission::CronUpdateParamsPermission { .. } => {
                PermissionType::CronUpdateParamsPermission
            }
            Permission::AllowAll => PermissionType::AllowAll,
            Permission::ParamChangePermission(_) => PermissionType::ParamChangePermission,
        }
    }
}

/// Checks that the strategy owner is authorised to change the parameters of the
/// cron module. We query the current values for each parameter & compare them to
/// the values in the proposal; all modifications must be allowed by the strategy.
fn check_cron_update_msg_params(
    deps: Deps,
    permission: CronUpdateParamsPermission,
    msg: MsgUpdateParamsCron,
) -> Result<(), ContractError> {
    let cron_params = get_cron_params(deps, ParamsRequestCron {})?;
    if cron_params.params.limit != msg.params.limit && !permission.limit {
        return Err(ContractError::Unauthorized {});
    }

    if cron_params.params.security_address != msg.params.security_address
        && !permission.security_address
    {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

fn check_param_change_permission(
    proposal: ParamChangeProposal,
    permissions: ParamChangePermission,
) -> Result<(), ContractError> {
    // TODO: keep perms as hashset in storage
    let perm: HashSet<ParamPermission> = HashSet::from_iter(permissions.params.into_iter());
    for param_change in proposal.param_changes {
        if perm
            .get(&ParamPermission {
                subspace: param_change.subspace,
                key: param_change.key,
            })
            .is_none()
        {
            return Err(ContractError::Unauthorized {});
        }
    }
    Ok(())
}
