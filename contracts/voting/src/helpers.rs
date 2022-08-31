use std::cmp::min;

use cosmwasm_std::Uint128;

use crate::msg::{PositionResponse, Schedule};
use crate::state::Position;

/// The return value is a three-tuple consists of: the vested amount, the unlocked amount, and the
/// withdrawable amount
pub fn compute_withdrawable(
    time: u64,
    total: Uint128,
    withdrawn: Uint128,
    vest_schedule: Schedule,
    unlock_schedule: Schedule,
) -> (Uint128, Uint128, Uint128) {
    let compute = |schedule: Schedule| {
        // before the end of cliff period, no token will be vested/unlocked
        if time < schedule.start_time + schedule.cliff {
            return Uint128::zero();
        }
        // after the duration, all tokens are fully vested/unlocked
        if time > schedule.start_time + schedule.duration {
            return total;
        }
        // otherwise, tokens vest/unlock linearly
        total.multiply_ratio(time - schedule.start_time, schedule.duration)
    };

    let vested = compute(vest_schedule);
    let unlocked = compute(unlock_schedule);

    let withdrawable =
        min(vested, unlocked).checked_sub(withdrawn).unwrap_or_else(|_| Uint128::zero());

    (vested, unlocked, withdrawable)
}

pub fn compute_position_response(
    time: u64,
    user: impl Into<String>,
    position: &Position,
    unlock_schedule: Schedule,
) -> PositionResponse {
    let (vested, unlocked, withdrawable) = compute_withdrawable(
        time,
        position.total,
        position.withdrawn,
        position.vest_schedule,
        unlock_schedule,
    );

    PositionResponse {
        user: user.into(),
        total: position.total,
        vested,
        unlocked,
        withdrawn: position.withdrawn,
        withdrawable,
        vest_schedule: position.vest_schedule,
    }
}
