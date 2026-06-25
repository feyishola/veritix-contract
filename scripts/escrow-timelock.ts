import {
  Contract,
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  nativeToScVal,
  Address,
} from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org";
const CONTRACT_ID = process.env.CONTRACT_ID ?? "";
const DEPOSITOR_SECRET = process.env.DEPOSITOR_SECRET ?? "";

async function getLedgerHeight(): Promise<number> {
  const server = new SorobanRpc.Server(RPC_URL);
  const latest = await server.getLatestLedger();
  return latest.sequence;
}

async function createTimeLockedEscrow(
  beneficiary: string,
  amount: bigint,
  lockLedgers: number
): Promise<string> {
  if (!CONTRACT_ID || !DEPOSITOR_SECRET) {
    throw new Error("CONTRACT_ID and DEPOSITOR_SECRET env vars are required");
  }

  const depositor = Keypair.fromSecret(DEPOSITOR_SECRET);
  const server = new SorobanRpc.Server(RPC_URL);
  const contract = new Contract(CONTRACT_ID);

  const currentLedger = await getLedgerHeight();
  const releaseAfterLedger = currentLedger + lockLedgers;
  console.log(`Current ledger: ${currentLedger}, release after: ${releaseAfterLedger}`);

  const account = await server.getAccount(depositor.publicKey());
  let tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(
      contract.call(
        "create_escrow",
        new Address(depositor.publicKey()).toScVal(),
        new Address(beneficiary).toScVal(),
        nativeToScVal(amount, { type: "i128" }),
        nativeToScVal(releaseAfterLedger, { type: "u32" })
      )
    )
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (!SorobanRpc.Api.isSimulationSuccess(sim)) {
    throw new Error(`Simulation failed: ${JSON.stringify(sim)}`);
  }

  tx = SorobanRpc.assembleTransaction(tx, sim).build();
  tx.sign(depositor);

  const response = await server.sendTransaction(tx);
  console.log("Time-locked escrow created:", response.hash);
  return response.hash;
}

const [beneficiary, amountStr, lockStr] = process.argv.slice(2);
createTimeLockedEscrow(beneficiary, BigInt(amountStr ?? "0"), parseInt(lockStr ?? "100", 10)).catch(
  (err) => {
    console.error(err.message);
    process.exit(1);
  }
);
