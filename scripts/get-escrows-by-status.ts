import { Contract, SorobanRpc, TransactionBuilder, Networks, BASE_FEE } from "@stellar/stellar-sdk";

const RPC_URL = "https://soroban-testnet.stellar.org";
const CONTRACT_ID = process.env.CONTRACT_ID ?? "";
const SOURCE_ACCOUNT = process.env.SOURCE_ACCOUNT ?? "";

interface EscrowStatusResult {
  active: number[];
  count: number;
}

async function getEscrowsByStatus(): Promise<EscrowStatusResult> {
  if (!CONTRACT_ID || !SOURCE_ACCOUNT) {
    throw new Error("CONTRACT_ID and SOURCE_ACCOUNT env vars are required");
  }

  const server = new SorobanRpc.Server(RPC_URL);
  const contract = new Contract(CONTRACT_ID);

  const account = await server.getAccount(SOURCE_ACCOUNT);
  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(contract.call("get_active_escrows"))
    .setTimeout(30)
    .build();

  const sim = await server.simulateTransaction(tx);
  if (!SorobanRpc.Api.isSimulationSuccess(sim)) {
    throw new Error(`Simulation failed: ${JSON.stringify(sim)}`);
  }

  const raw = (sim.result?.retval as any)?.value ?? [];
  const active: number[] = raw.map((v: any) => Number(v.value));

  return { active, count: active.length };
}

async function main() {
  const result = await getEscrowsByStatus();
  console.log(`Active escrows (${result.count}):`, result.active);
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
