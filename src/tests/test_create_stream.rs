#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Address;

    use crate::contract::StreamFundContractClient;
    use crate::errors::StreamError;
    use crate::tests::helpers::{
        balance, create_default_stream, set_time, setup, AMOUNT, DURATION, START_OFFSET,
    };

    // ── 6.1: Happy path ──────────────────────────────────────────────────────

    #[test]
    fn create_stream_happy_path() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);

        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &AMOUNT, &start, &stop);
        assert!(result.is_ok());

        assert_eq!(balance(&env, &token_id, &contract_id), AMOUNT);
        assert_eq!(balance(&env, &token_id, &sender), AMOUNT * 10 - AMOUNT);
    }

    // ── 6.2: Duplicate stream_id ─────────────────────────────────────────────

    #[test]
    fn create_stream_duplicate_id() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &AMOUNT, &start, &stop);
        assert_eq!(result.unwrap_err(), StreamError::DuplicateStreamId);
    }

    // ── 6.3: Invalid time range ───────────────────────────────────────────────

    #[test]
    fn create_stream_start_equals_stop() {
        let (env, contract_id, token_id, sender, receiver, _now, start, _stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &AMOUNT, &start, &start);
        assert_eq!(result.unwrap_err(), StreamError::InvalidTimeRange);
    }

    #[test]
    fn create_stream_start_after_stop() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &AMOUNT, &stop, &start);
        assert_eq!(result.unwrap_err(), StreamError::InvalidTimeRange);
    }

    // ── 6.4: Stop time in past ────────────────────────────────────────────────

    #[test]
    fn create_stream_stop_time_in_past() {
        let (env, contract_id, token_id, sender, receiver, now, _start, _stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let past_stop = now - 1;
        let past_start = past_stop - DURATION;
        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &AMOUNT, &past_start, &past_stop);
        assert_eq!(result.unwrap_err(), StreamError::StopTimeInPast);
    }

    // ── 6.5: Invalid amount ───────────────────────────────────────────────────

    #[test]
    fn create_stream_zero_amount() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &0, &start, &stop);
        assert_eq!(result.unwrap_err(), StreamError::InvalidAmount);
    }

    #[test]
    fn create_stream_negative_amount() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let result = client.create_stream(&1u64, &sender, &receiver, &token_id, &(-1), &start, &stop);
        assert_eq!(result.unwrap_err(), StreamError::InvalidAmount);
    }

    // ── 6.6: Self-stream ──────────────────────────────────────────────────────

    #[test]
    fn create_stream_self_stream() {
        let (env, contract_id, token_id, sender, _receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let result = client.create_stream(&1u64, &sender, &sender, &token_id, &AMOUNT, &start, &stop);
        assert_eq!(result.unwrap_err(), StreamError::SelfStream);
    }

    // ── 6.7: Authorization failure ────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn create_stream_no_auth() {
        let env = soroban_sdk::Env::default();
        // No mock_all_auths — require_auth will panic.
        let contract_id = env.register(crate::contract::StreamFundContract, ());
        let client = StreamFundContractClient::new(&env, &contract_id);
        let sender = Address::generate(&env);
        let receiver = Address::generate(&env);
        let token = Address::generate(&env);
        let now = env.ledger().timestamp();
        let _ = client.create_stream(
            &1u64, &sender, &receiver, &token, &AMOUNT,
            &(now + START_OFFSET), &(now + START_OFFSET + DURATION),
        );
    }
}
