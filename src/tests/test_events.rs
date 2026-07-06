#[cfg(test)]
mod tests {
    use soroban_sdk::testutils::Events;
    use soroban_sdk::{symbol_short, vec, IntoVal};

    use crate::contract::StreamFundContractClient;
    use crate::tests::helpers::{create_default_stream, set_time, setup, AMOUNT, DURATION};

    // ── Verify stream_created event topics and data ───────────────────────────

    #[test]
    fn event_stream_created_correct() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        let events = env.events().all();
        // The last event emitted should be stream_created.
        let (contract, topics, data) = events.last().unwrap();

        assert_eq!(contract, contract_id);
        assert_eq!(
            topics,
            vec![
                &env,
                symbol_short!("created").into_val(&env),
                1u64.into_val(&env),
            ]
        );
        // Data tuple: (sender, receiver, token, amount, start_time, stop_time)
        let expected_data = (
            sender.clone(),
            receiver.clone(),
            token_id.clone(),
            AMOUNT,
            start,
            stop,
        )
            .into_val(&env);
        assert_eq!(data, expected_data);
    }

    // ── Verify stream_withdrawn event ─────────────────────────────────────────

    #[test]
    fn event_stream_withdrawn_correct() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);
        set_time(&env, stop);
        client.withdraw(&id, &AMOUNT).unwrap();

        let events = env.events().all();
        let (contract, topics, data) = events.last().unwrap();

        assert_eq!(contract, contract_id);
        assert_eq!(
            topics,
            vec![
                &env,
                symbol_short!("withdrawn").into_val(&env),
                id.into_val(&env),
            ]
        );
        let expected_data = (receiver.clone(), AMOUNT).into_val(&env);
        assert_eq!(data, expected_data);
    }

    // ── Verify stream_cancelled event ─────────────────────────────────────────

    #[test]
    fn event_stream_cancelled_correct() {
        let (env, contract_id, token_id, sender, receiver, _now, start, stop) = setup();
        let client = StreamFundContractClient::new(&env, &contract_id);
        let id = create_default_stream(&client, &token_id, &sender, &receiver, start, stop);

        // Cancel at halfway — sender_refund = AMOUNT/2, receiver_payout = AMOUNT/2
        set_time(&env, start + DURATION / 2);
        client.cancel_stream(&id).unwrap();

        let events = env.events().all();
        let (contract, topics, data) = events.last().unwrap();

        assert_eq!(contract, contract_id);
        assert_eq!(
            topics,
            vec![
                &env,
                symbol_short!("cancelled").into_val(&env),
                id.into_val(&env),
            ]
        );
        let expected_data = (sender.clone(), AMOUNT / 2_i128, AMOUNT / 2_i128).into_val(&env);
        assert_eq!(data, expected_data);
    }
}
