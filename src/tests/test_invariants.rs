#[cfg(test)]
mod tests {
    use crate::contract::StreamFundContractClient;
    use crate::tests::helpers::{balance, create_default_stream, set_time, setup, AMOUNT, DURATION};

    // ── 10.1: create → partial withdraw → cancel — conservation ──────────────
    //
    // At cancel midpoint (t = start + DURATION/2):
    //   streamed        = AMOUNT / 2
    //   withdrawn       = AMOUNT / 4  (half of what was available)
    //   receiver_retained = AMOUNT/2 - AMOUNT/4 = AMOUNT/4
    //   sender_refund     = AMOUNT - AMOUNT/2   = AMOUNT/2
    //   CHECK: withdrawn + retained + refund = AMOUNT/4 + AMOUNT/4 + AMOUNT/2 = AMOUNT ✓

    #[test]
    fn invariant_partial_withdraw_then_cancel() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        set_time(&env, start + DURATION / 2);
        client.withdraw(&id, &(AMOUNT / 4)).unwrap();
        client.cancel_stream(&id).unwrap();

        // sender got AMOUNT/2 back (unstreamed portion)
        // sender deposited AMOUNT, so net balance = AMOUNT*10 - AMOUNT + AMOUNT/2
        let expected_sender = AMOUNT * 10 - AMOUNT + AMOUNT / 2;
        assert_eq!(balance(&env, &token_id, &sender), expected_sender);

        // receiver has withdrawn AMOUNT/4, retained AMOUNT/4 still in contract
        assert_eq!(balance(&env, &token_id, &receiver), AMOUNT / 4);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT / 4);

        // contract holds the retained amount only
        assert_eq!(balance(&env, &token_id, &contract_id), AMOUNT / 4);
    }

    // ── 10.2: create → cancel → full withdraw — contract empties ─────────────

    #[test]
    fn invariant_cancel_then_full_withdraw_empties_contract() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();

        let retained = client.withdrawable_balance(&id).unwrap();
        client.withdraw(&id, &retained).unwrap();

        assert_eq!(balance(&env, &token_id, &contract_id), 0);
    }

    // ── 10.3: create → withdraw full at stop_time ─────────────────────────────

    #[test]
    fn invariant_withdraw_full_at_stop_time() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        set_time(&env, stop);
        assert_eq!(client.withdrawable_balance(&id).unwrap(), AMOUNT);
        client.withdraw(&id, &AMOUNT).unwrap();

        assert_eq!(balance(&env, &token_id, &contract_id), 0);
        assert_eq!(balance(&env, &token_id, &receiver), AMOUNT);
    }

    // ── 10.4: withdrawable_balance is never negative at any time point ────────

    #[test]
    fn invariant_balance_never_negative() {
        let (env, contract_id, token_id, sender, receiver, now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        let time_points = [
            now,
            start - 1,
            start,
            start + DURATION / 4,
            start + DURATION / 2,
            start + (DURATION * 3 / 4),
            stop,
            stop + 1_000,
        ];

        for &t in &time_points {
            set_time(&env, t);
            let bal = client.withdrawable_balance(&id).unwrap();
            assert!(bal >= 0, "balance was negative ({bal}) at t={t}");
        }
    }

    // ── 10.5: withdrawn_amount is monotonically non-decreasing ────────────────

    #[test]
    fn invariant_withdrawn_amount_monotonic() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        set_time(&env, stop);

        // AMOUNT / 5 = 200_000 — divides evenly
        let chunk = AMOUNT / 5;
        let mut total_withdrawn = 0i128;

        for _ in 0..5 {
            client.withdraw(&id, &chunk).unwrap();
            total_withdrawn += chunk;
            let available = client.withdrawable_balance(&id).unwrap();
            assert_eq!(available, AMOUNT - total_withdrawn);
            assert!(available >= 0);
        }

        assert_eq!(total_withdrawn, AMOUNT);
        assert_eq!(balance(&env, &token_id, &receiver), AMOUNT);
    }

    // ── Bonus: full token conservation across a complete lifecycle ────────────
    //
    // Initial supply: AMOUNT * 10 (minted to sender)
    // After create_stream: sender loses AMOUNT → contract gains AMOUNT
    // At t = start + DURATION/3: receiver withdraws available, sender cancels
    // Final: sender + receiver + contract == AMOUNT * 10

    #[test]
    fn invariant_no_tokens_created_or_destroyed() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        set_time(&env, start + DURATION / 3);

        let available = client.withdrawable_balance(&id).unwrap();
        client.withdraw(&id, &available).unwrap();
        client.cancel_stream(&id).unwrap();

        let total = balance(&env, &token_id, &sender)
            + balance(&env, &token_id, &receiver)
            + balance(&env, &token_id, &contract_id);

        assert_eq!(total, AMOUNT * 10, "token conservation violated: total={total}");
    }
}
