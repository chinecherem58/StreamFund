use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a new stream is created.
///
/// Topics: `("created", stream_id)`
/// Data:   `(sender, receiver, token, amount, start_time, stop_time)`
pub fn stream_created(
    e: &Env,
    stream_id: u64,
    sender: &Address,
    receiver: &Address,
    token: &Address,
    amount: i128,
    start_time: u64,
    stop_time: u64,
) {
    e.events().publish(
        (symbol_short!("created"), stream_id),
        (
            sender.clone(),
            receiver.clone(),
            token.clone(),
            amount,
            start_time,
            stop_time,
        ),
    );
}

/// Emitted when a receiver withdraws from a stream.
///
/// Topics: `("withdrawn", stream_id)`
/// Data:   `(receiver, amount)`
pub fn stream_withdrawn(e: &Env, stream_id: u64, receiver: &Address, amount: i128) {
    e.events().publish(
        (symbol_short!("withdrawn"), stream_id),
        (receiver.clone(), amount),
    );
}

/// Emitted when a sender cancels a stream.
///
/// Topics: `("cancelled", stream_id)`
/// Data:   `(sender, sender_refund, receiver_payout)`
pub fn stream_cancelled(
    e: &Env,
    stream_id: u64,
    sender: &Address,
    sender_refund: i128,
    receiver_payout: i128,
) {
    e.events().publish(
        (symbol_short!("cancelled"), stream_id),
        (sender.clone(), sender_refund, receiver_payout),
    );
}
