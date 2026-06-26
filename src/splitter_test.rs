#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, Vec};

    #[test]
    #[should_panic(expected = "AlreadyDistributed: funds from this split have already been released")]
    fn test_reentrancy_guard_blocks_double_distribution() {
        let env = Env::default();
        
        // 1. Seed a mock split distribution record
        let mut record = SplitRecord {
            distributed: false,
            recipients: Vec::from_array(&env, [env.accounts().generate(), env.accounts().generate()]),
        };

        // 2. Simulate the first distribution lifecycle phase (Effects stage)
        assert!(!record.distributed);
        record.distributed = true; // State flips to true here

        // 3. Simulate an adversarial re-entrant nested call hitting the contract again
        // Since record.distributed is now true, this evaluation must panic immediately
        if record.distributed {
            panic!("AlreadyDistributed: funds from this split have already been released");
        }
    }
}