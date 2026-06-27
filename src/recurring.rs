// ─── Before / Buggy Implementation ───────────────────────────────────────
// If execution is delayed, this drifts the schedule forward, giving away free ledgers.
// record.last_charged_ledger = e.ledger().sequence();

// ─── After / Correct Implementation ────────────────────────────────────────
// Anchors the schedule back onto the original baseline setup interval alignment.
record.last_charged_ledger = record
    .last_charged_ledger
    .checked_add(record.interval)
    .expect("Overflow calculating next structural billing execution ledger baseline");