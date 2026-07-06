use soroban_sdk::Env;

use crate::errors::StreamError;
use crate::types::{StorageKey, Stream};

/// Minimum TTL bump applied on every write (roughly 1 day at ~5s/ledger).
const MIN_TTL_LEDGERS: u32 = 17_280;

/// Load a stream from persistent storage.
///
/// Returns `StreamError::StreamNotFound` if no record exists for `stream_id`.
pub fn load_stream(e: &Env, stream_id: u64) -> Result<Stream, StreamError> {
    let key = StorageKey::Stream(stream_id);
    e.storage()
        .persistent()
        .get(&key)
        .ok_or(StreamError::StreamNotFound)
}

/// Persist a stream and extend its TTL.
///
/// TTL is extended to at least `(stop_time - current_time) + 31_536_000`
/// ledger-seconds beyond the current ledger, capped so it never underflows.
pub fn save_stream(e: &Env, stream_id: u64, stream: &Stream) {
    let key = StorageKey::Stream(stream_id);
    e.storage().persistent().set(&key, stream);

    let current_time = e.ledger().timestamp();
    let ttl_seconds = stream
        .stop_time
        .saturating_sub(current_time)
        .saturating_add(31_536_000_u64);

    // Soroban TTL is expressed in ledgers. Stellar mainnet targets ~5s/ledger.
    // We use a safe upper-bound of 5s per ledger for the conversion.
    let ttl_ledgers = (ttl_seconds / 5).min(u32::MAX as u64) as u32;
    let max_ttl = ttl_ledgers.max(MIN_TTL_LEDGERS);

    e.storage()
        .persistent()
        .extend_ttl(&key, MIN_TTL_LEDGERS, max_ttl);
}

/// Check whether a stream_id is already occupied in storage.
pub fn stream_exists(e: &Env, stream_id: u64) -> bool {
    let key = StorageKey::Stream(stream_id);
    e.storage().persistent().has(&key)
}
