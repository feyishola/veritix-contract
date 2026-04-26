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
const ADMIN_SECRET = process.env.ADMIN_SECRET ?? "";

async function forceRefundEscrow(escrowId: number): Promise<string> {
  if (!CONTRACT_ID || !ADMIN_SECRET) {
    throw new Error("CONTRACT_ID and ADMIN_SECRET env vars are required");
  }

  const admin = Keypair.fromSecret(ADMIN_SECRET);
  const server = new SorobanRpc.Server(RPC_URL);
  const contract = new Contract(CONTRACT_ID);

  const account = await server.getAccount(admin.publicKey());
  let tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(
      contract.call(
        "force_refund_escrow",
        new Address(admin.publicKey()).toScVal(),
        nativeToScVal(escrowId, { type: "u32" })
      )
    )
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (!SorobanRpc.Api.isSimulationSuccess(sim)) {
    throw new Error(`Simulation failed: ${JSON.stringify(sim)}`);
  }

  tx = SorobanRpc.assembleTransaction(tx, sim).build();
  tx.sign(admin);

  const response = await server.sendTransaction(tx);
  console.log(`Force refund submitted for escrow #${escrowId}:`, response.hash);
  return response.hash;
}

const escrowId = parseInt(process.argv[2] ?? "0", 10);
forceRefundEscrow(escrowId).catch((err) => {
  console.error(err.message);
  process.exit(1);
});
