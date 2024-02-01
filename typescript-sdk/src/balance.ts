import { UNION_RPC_URL } from "#/constants";
import { erc20Abi, type Address } from "viem";
import type { UnionClient } from "#/actions.ts";
import { chainIds } from "#/constants/chain.ts";
import { StargateClient } from "@cosmjs/stargate";

export type GetBalanceParameters =
  | ({ chainId: "32382" } & GetBalanceOnUnion)
  | ({ chainId: "1" | "11155111" } & GetBalanceOnEthereum);

export async function getBalance(
  client: UnionClient,
  args: GetBalanceParameters
): Promise<bigint> {
  if (!chainIds.includes(args.chainId))
    throw new Error(`Invalid chainId: ${args.chainId}`);
  if (args.chainId === "32382") return  getBalanceOnUnion(args);
  return  getBalanceOnEthereum(client, args);
}

interface GetBalanceOnUnion {
  address: string;
  assetId: string;
  unionRpcUrl?: string;
}

async function getBalanceOnUnion({
  address,
  assetId = "muno",
  unionRpcUrl = UNION_RPC_URL,
}: GetBalanceOnUnion): Promise<bigint> {
  const client = await StargateClient.connect(unionRpcUrl);
  const { amount } = await client.getBalance(address, assetId);
  return BigInt(amount);
}

interface GetBalanceOnEthereum {
  address: Address;
  assetId: Address;
}

async function getBalanceOnEthereum(
  client: UnionClient,
  { address, assetId }: GetBalanceOnEthereum
): Promise<bigint> {
  return  client.readContract({
    abi: erc20Abi,
    functionName: "balanceOf",
    address: assetId,
    args: [address],
  });
}
