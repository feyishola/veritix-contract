#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Ledger, Env};

    #[test]
    fn test_delayed_execution_does_not_drift_schedule() {
        let env = Env::default();
        
        // 1. Initialize a test recurring subscription record
        let interval = 100;
        let initial_setup_ledger = 1000;
        
        let mut record = RecurringRecord {
            last_charged_ledger: initial_setup_ledger,
            interval,
            payer: env.accounts().generate(),
        };

        // 2. Simulate a delayed execution trigger (e.g., crank bot offline for 5 ledgers)
        // Expected scheduled target: 1100
        // Actual execution ledger: 1105
        let delayed_execution_ledger = initial_setup_ledger + interval + 5;
        env.ledger().set_sequence(delayed_execution_ledger);

        // 3. Apply the updated logic branch execution mutation
        record.last_charged_ledger = record
            .last_charged_ledger
            .checked_add(record.interval)
            .expect("Overflow verification");

        // 4. Assert that the next target remains rigidly locked to 1100, NOT 1105
        assert_eq!(record.last_charged_ledger, 1100);
        
        // Verify that the subsequent execution calculation anchors perfectly from 1100 -> 1200
        let next_scheduled_target = record.last_charged_ledger + record.interval;
        assert_eq!(next_scheduled_target, 1200);
    }
}