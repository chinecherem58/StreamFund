#[cfg(test)]
mod tests {
    use crate::contract::StreamFundContractClient;
    use crate::errors::StreamError;
    use crate::tests::helpers::{create_default_stream, set_time, setup, AMOUNT, DURATION};

    // ── 7.1: t < start_time ───────────────────────────────────────────────────

    #[test]
    fn balance_before_start_returns_zero() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        // still before start_time
        assert_eq!(client.withdrawable_balance(&id).unwrap(), 0);
    }

    // ── 7.2: t == start_time ──────────────────────────────────────────────────

    #[test]
    fn balance_at_start_returns_zero() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), 0);
    }

    // ── 7.3: t midway ─────────────────────────────────────────────────────────

    #[test]
    fn balance_midway_correct() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);
        let expected = AMOUNT * (DURATION as i128 / 2) / DURATION as i128;
        assert_eq!(client.withdrawable_balance(&id).unwrap(), expected);
    }

    #[test]
    fn balance_quarter_point() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 4);
        let expected = AMOUNT * (DURATION as i128 / 4) / DURATION as i128;
        assert_eq!(client.withdrawable_balance(&id).unwrap(), expected);
    }

    // ── 7.4: t == stop_time ───────────────────────────────────────────────────

    #[test]
    fn balance_at_stop_time() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT);
    }

    // ── 7.5: t > stop_time ────────────────────────────────────────────────────

    #[test]
    fn balance_after_stop_time() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop + 10_000);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT);
    }

    // ── 7.6: After partial withdrawal ─────────────────────────────────────────

    #[test]
    fn balance_after_partial_withdrawal() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        client.withdraw(&id, &(AMOUNT / 2)).unwrap();
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT / 2);
    }

    // ── 7.7: Cancelled stream returns receiver_retained ───────────────────────

    #[test]
    fn balance_cancelled_stream_returns_retained() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();
        // should return captured retained balance, not a new time-based value
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT / 2);
    }

    // ── 7.8: StreamNotFound ───────────────────────────────────────────────────

    #[test]
    fn balance_stream_not_found() {
        let (env, contract_id, _token_id, _sender, _receiver, _now, _start, _stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        assert_eq!(client.withdrawable_balance(&999u64).unwrap_err(), StreamError::StreamNotFound);
    }
}
