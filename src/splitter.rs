// ─── Before / Vulnerable Layout ──────────────────────────────────────────
// Recipient contract calls back into distribute() before state is updated,
// leading to duplicate token drains.
//
// for recipient in record.recipients.iter() { ... }
// record.distributed = true;

// ─── After / Secure Layout (Checks-Effects-Interactions Pattern) ──────────
// 1. Checks: Ensure the record hasn't been drained already
if record.distributed {
    panic!("AlreadyDistributed: funds from this split have already been released");
}

// 2. Effects: Mutate state IMMEDIATELY before external interactions to block re-entrancy
// Intentional ordering: This blocks nested contract callbacks from double-dipping.
record.distributed = true;
env.storage().persistent().set(&DataKey::SplitRecord(split_id), &record);

// 3. Interactions: Securely proceed with external transfer loops
for recipient in record.recipients.iter() {
    // Cross-contract calls / transfers execute safely here...
}