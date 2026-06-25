import {
  Contract,
  Keypair,
  SorobanRpc,
  TransactionBuilder,
  Networks,
  BASE_FEE,
  Address,
} from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org";
const CONTRACT_ID = process.env.CONTRACT_ID ?? "";
const ADMIN_SECRET = process.env.ADMIN_SECRET ?? "";

// Verifies that freeze/unfreeze only requires a single admin auth at the entrypoint.
// After the redundant require_auth() removal in freeze.rs, simulation should show
// exactly one auth entry — the admin's — not two.
async function verifyFreezeAuthCount(target: string): Promise<void> {
  if (!CONTRACT_ID || !ADMIN_SECRET) {
    throw new Error("CONTRACT_ID and ADMIN_SECRET env vars are required");
  }

  const admin = Keypair.fromSecret(ADMIN_SECRET);
  const server = new SorobanRpc.Server(RPC_URL);
  const contract = new Contract(CONTRACT_ID);

  const account = await server.getAccount(admin.publicKey());

  for (const op of ["freeze_account", "unfreeze_account"] as const) {
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: Networks.TESTNET,
    })
      .addOperation(contract.call(op, new Address(admin.publicKey()).toScVal(), new Address(target).toScVal()))
      .setTimeout(30)
      .build();

    const sim = await server.simulateTransaction(tx);
    if (!SorobanRpc.Api.isSimulationSuccess(sim)) {
      console.error(`${op} simulation failed:`, JSON.stringify(sim));
      continue;
    }

    const authCount = sim.result?.auth?.length ?? 0;
    const status = authCount === 1 ? "✓ single auth" : `✗ unexpected auth count: ${authCount}`;
    console.log(`${op}: ${status}`);
  }
}

const target = process.argv[2] ?? "";
if (!target) {
  console.error("Usage: ts-node verify-freeze-auth.ts <target-account>");
  process.exit(1);
}
verifyFreezeAuthCount(target).catch((err) => {
  console.error(err.message);
  process.exit(1);
});
