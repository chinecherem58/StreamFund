#[cfg(test)]
mod tests {
    use crate::contract::StreamFundContractClient;
    use crate::errors::StreamError;
    use crate::tests::helpers::{balance, create_default_stream, set_time, setup, AMOUNT, DURATION};

    // ── 9.1: Cancel before start — full refund ───────────────────────────────

    #[test]
    fn cancel_before_start_full_refund() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        let before = balance(&env, &token_id, &sender);
        client.cancel_stream(&id).unwrap();
        assert_eq!(balance(&env, &token_id, &sender) - before, AMOUNT);

        // receiver_retained is 0 — withdrawal should fail
        assert_eq!(client.withdraw(&id, &1).unwrap_err(), StreamError::InsufficientBalance);
    }

    // ── 9.2: Cancel midway — correct accounting split ─────────────────────────

    #[test]
    fn cancel_midway_correct_split() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);

        let before = balance(&env, &token_id, &sender);
        client.cancel_stream(&id).unwrap();
        assert_eq!(balance(&env, &token_id, &sender) - before, AMOUNT / 2);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT / 2);
    }

    // ── 9.3: Cancel at/after stop — StreamNotActive ───────────────────────────

    #[test]
    fn cancel_after_stop_is_not_active() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop + 1);
        assert_eq!(client.cancel_stream(&id).unwrap_err(), StreamError::StreamNotActive);
    }

    // ── 9.4: Event emission does not panic ────────────────────────────────────

    #[test]
    fn cancel_emits_event_no_panic() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 4);
        client.cancel_stream(&id).unwrap();
    }

    // ── 9.5: StreamNotActive on already-cancelled stream ─────────────────────

    #[test]
    fn cancel_already_cancelled() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        client.cancel_stream(&id).unwrap();
        assert_eq!(client.cancel_stream(&id).unwrap_err(), StreamError::StreamNotActive);
    }

    // ── 9.6: StreamNotFound ───────────────────────────────────────────────────

    #[test]
    fn cancel_stream_not_found() {
        let (env, contract_id, _token_id, _sender, _receiver, _now, _start, _stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        assert_eq!(client.cancel_stream(&999u64).unwrap_err(), StreamError::StreamNotFound);
    }

    // ── 9.7: Authorization failure ────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn cancel_no_auth() {
        let env = soroban_sdk::Env::default();
        // No mock_all_auths — sender require_auth will panic.
        let contract_id = env.register(crate::contract::StreamFundContract, ());
        let client = StreamFundContractClient::new(&env, &contract_id);
        let _ = client.cancel_stream(&1u64);
    }

    // ── 9.8: Receiver withdraws retained balance after cancel ─────────────────

    #[test]
    fn cancel_receiver_can_withdraw_retained() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();

        let before = balance(&env, &token_id, &receiver);
        client.withdraw(&id, &(AMOUNT / 2)).unwrap();
        assert_eq!(balance(&env, &token_id, &receiver) - before, AMOUNT / 2);
    }
}
