use soroban_sdk::contracterror;

/// All error conditions surfaced by `StreamFundContract`.
///
/// Variants are assigned stable integer discriminants so on-chain error codes
/// remain consistent across contract upgrades.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StreamError {
    /// No stream exists for the provided `stream_id`.
    StreamNotFound = 1,
    /// A stream with this `stream_id` already exists in storage.
    DuplicateStreamId = 2,
    /// `start_time >= stop_time`.
    InvalidTimeRange = 3,
    /// `stop_time` is at or before the current ledger timestamp.
    StopTimeInPast = 4,
    /// `amount` is zero or negative.
    InvalidAmount = 5,
    /// `sender` and `receiver` are the same address.
    SelfStream = 6,
    /// Requested withdrawal exceeds the available balance.
    InsufficientBalance = 7,
    /// Operation requires an Active stream but the stream is Cancelled or Completed.
    StreamNotActive = 8,
    /// An outbound token transfer failed.
    TransferFailed = 9,
    /// The caller is not authorised to perform this operation.
    Unauthorized = 10,
}
