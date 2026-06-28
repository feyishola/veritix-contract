# VeriTix Whitepaper Outline

## Problem

The live event ticketing industry suffers from systemic failures:

- **Ticket fraud**: Counterfeit tickets, duplicate barcodes, and unauthorized resale
- **Scalping**: Bots and bulk buyers capture inventory at face value, reselling at extreme markups
- **Opaque revenue splits**: Event organizers, artists, and venues have limited visibility into how ticket revenue is distributed
- **Settlement delays**: Payouts from primary ticket sellers can take weeks or months to reach organizers, artists, and venues
- **Dispute resolution**: No transparent mechanism for handling refunds, cancellations, or chargebacks

## Solution: On-Chain Ticket Tokens

VeriTix issues tokenized tickets on the Stellar network using Soroban smart contracts. Each ticket is a non-divisible token with programmable rules for transfer, escrow, and settlement.

Key primitives:

- **Escrow**: Funds for ticket purchases are held in escrow until the event occurs or a refund condition is met
- **Dispute resolution**: A transparent on-chain process for resolving conflicts between ticket buyers, sellers, and event organizers
- **Payment splitting**: Ticket revenue is automatically split between organizers, artists, venues, and other stakeholders in configurable proportions

## Token Economics

The VTX utility token powers the VeriTix ecosystem:

- **Issuance**: Tokens are minted by the contract admin according to a predefined schedule
- **Utility**: VTX is used for ticket purchases, platform fees, and governance participation
- **Burn mechanics**: A portion of platform fees may be burned to create deflationary pressure
- **Max supply**: A configurable cap on total supply prevents unlimited inflation

## Governance

The current governance model uses a two-tier admin system:

- **Admin**: Full control over minting, freezing, clawback, and contract parameters
- **Cosigner**: Required for sensitive operations like clawback, providing a checks-and-balances mechanism
- **Admin rotation**: The admin can propose a new admin address; the proposed address must accept

**Future path**: A DAO-based governance model where VTX token holders vote on protocol parameters, fee structures, and upgrades.

## Protocol Revenue

Platform fees flow through the splitter module:

1. A percentage of each escrow release is collected as a protocol fee
2. Fees are distributed to configured recipients (platform treasury, ecosystem fund, etc.)
3. Fee distribution uses the same splitter mechanism as standard payments

## Roadmap

### Current Wave

- Core token operations (mint, burn, transfer, approve)
- Escrow lifecycle (create, release, refund, admin settle)
- Dispute resolution (open, resolve, expire)
- Payment splitting (create, distribute, cancel)
- Recurring payments (setup, execute, cancel)
- Admin controls (rotation, pause, freeze, clawback)

### Planned Enhancements

- Whitelist mode for restricted transfers
- Vesting schedules for token distribution
- Protocol fee mechanism for sustainable revenue
- DAO governance for decentralized control
- SEP-41 interface compliance
- Batch operations for improved throughput
- Event-driven monitoring and alerting

---

*This document is a living outline and will be expanded as the protocol evolves.*
