#[cfg(test)]
mod tests {
    use crate::contract::StreamFundContractClient;
    use crate::errors::StreamError;
    use crate::tests::helpers::{balance, create_default_stream, set_time, setup, AMOUNT, DURATION};

    // ── 8.1: Happy path ───────────────────────────────────────────────────────

    #[test]
    fn withdraw_happy_path() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);

        let before = balance(&env, &token_id, &receiver);
        client.withdraw(&id, &AMOUNT).unwrap();
        assert_eq!(balance(&env, &token_id, &receiver) - before, AMOUNT);
        assert_eq!(balance(&env, &token_id, &contract_id), 0);
    }

    // ── 8.2: Multiple partial withdrawals ────────────────────────────────────

    #[test]
    fn withdraw_multiple_partial() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);

        client.withdraw(&id, &(AMOUNT / 4)).unwrap();
        client.withdraw(&id, &(AMOUNT / 4)).unwrap();
        client.withdraw(&id, &(AMOUNT / 2)).unwrap();

        assert_eq!(balance(&env, &token_id, &receiver), AMOUNT);
        assert_eq!(balance(&env, &token_id, &contract_id), 0);
    }

    // ── 8.3: Post-cancel withdrawal ───────────────────────────────────────────

    #[test]
    fn withdraw_after_cancellation() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();

        let before = balance(&env, &token_id, &receiver);
        client.withdraw(&id, &(AMOUNT / 2)).unwrap();
        assert_eq!(balance(&env, &token_id, &receiver) - before, AMOUNT / 2);
    }

    // ── 8.4: InsufficientBalance ─────────────────────────────────────────────

    #[test]
    fn withdraw_insufficient_balance() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        assert_eq!(client.withdraw(&id, &(AMOUNT + 1)).unwrap_err(), StreamError::InsufficientBalance);
    }

    #[test]
    fn withdraw_before_start_insufficient() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        // still before start_time — nothing available
        assert_eq!(client.withdraw(&id, &1).unwrap_err(), StreamError::InsufficientBalance);
    }

    // ── 8.5: InvalidAmount ────────────────────────────────────────────────────

    #[test]
    fn withdraw_zero_amount() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        assert_eq!(client.withdraw(&id, &0).unwrap_err(), StreamError::InvalidAmount);
    }

    #[test]
    fn withdraw_negative_amount() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        assert_eq!(client.withdraw(&id, &(-1)).unwrap_err(), StreamError::InvalidAmount);
    }

    // ── 8.6: StreamNotFound ───────────────────────────────────────────────────

    #[test]
    fn withdraw_stream_not_found() {
        let (env, contract_id, _token_id, _sender, _receiver, _now, _start, _stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        assert_eq!(client.withdraw(&999u64, &1).unwrap_err(), StreamError::StreamNotFound);
    }

    // ── 8.7: Authorization failure ────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn withdraw_no_auth() {
        let env = soroban_sdk::Env::default();
        // No mock_all_auths — receiver require_auth will panic.
        let contract_id = env.register(crate::contract::StreamFundContract, ());
        let client = StreamFundContractClient::new(&env, &contract_id);
        let _ = client.withdraw(&1u64, &1);
    }

    // ── 8.8: Exhausted retained balance ──────────────────────────────────────

    #[test]
    fn withdraw_exhausted_retained_balance() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();
        client.withdraw(&id, &(AMOUNT / 2)).unwrap();
        assert_eq!(client.withdraw(&id, &1).unwrap_err(), StreamError::InsufficientBalance);
    }
}
