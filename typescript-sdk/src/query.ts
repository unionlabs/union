import { ucs01relayAbi } from "./abi";
import { erc20Abi, type Address } from "viem";
import type { UnionClient } from "./actions.ts";
import { chainIds } from "./constants/chain.ts";
import { StargateClient } from "@cosmjs/stargate";
import { UCS01_EVM_ADDRESS, UNION_RPC_URL } from "./constants";
import { fetcher, raise } from "./utilities";

/**
 * Contract arguments:
 * @reference https://github.com/unionlabs/union/blob/a4190653d7ae91ea11221209b465ad0692616a3d/evm/contracts/apps/ucs/01-relay/Relay.sol#L31-L35
 * - 'sourcePort string'
 * - `sourceChannel string`
 * - 'denom string'
 */
export async function getDenomAddress(client: UnionClient): Promise<Address> {
  const [sourcePort, sourceChannel, denom] = [
    process.env.UCS01_SEPOLIA_PORT_ID || "ucs01-relay",
    process.env.UCS01_SEPOLIA_SOURCE_CHANNEL || "channel-0",
    process.env.UNION_NATIVE_DENOM || "muno",
  ];
  const UNION_CONTRACT_ADDRESS =
    process.env.UCS01_UNION_ADDRESS ||
    "union14pfzjnvzacqsmgjyf0avksc8cr70hsyt5epzcp66tmjpswf8sq8sn5meuy";

  return client.readContract({
    abi: ucs01relayAbi,
    address: UCS01_EVM_ADDRESS || "0x7f7AC7d5a1a2bD54dBA53a22209C3f96699Ed63c",
    functionName: "getDenomAddress",
    args: [
      sourcePort,
      sourceChannel,
      `wasm.${UNION_CONTRACT_ADDRESS}/${sourceChannel}/${denom}`,
    ],
  });
}

export type GetBalanceParameters =
  | ({ chainId: "6" } & GetBalanceOnUnion)
  | ({ chainId: "1" | "11155111" } & GetBalanceOnEthereum);

export async function getBalance(
  client: UnionClient,
  args: GetBalanceParameters
): Promise<bigint> {
  if (!chainIds.includes(args.chainId))
    throw new Error(`Invalid chainId: ${args.chainId}`);
  if (args.chainId === "6") return getBalanceOnUnion(args);
  return getBalanceOnEthereum(client, args);
}

interface GetBalanceOnEthereum {
  address: Address;
}

export async function getBalanceOnEthereum(
  client: UnionClient,
  { address }: GetBalanceOnEthereum
): Promise<bigint> {
  const denomAddress = await getDenomAddress(client);
  if (BigInt(denomAddress) === 0n) return 0n;

  return client.readContract({
    abi: erc20Abi,
    functionName: "balanceOf",
    address: denomAddress,
    args: [address],
  });
}

interface GetBalanceOnUnion {
  address: string;
  assetId?: string;
  unionRpcUrl?: string;
}

export async function getBalanceOnUnion({
  address,
  assetId = "muno",
  unionRpcUrl = UNION_RPC_URL || "https://union-testnet-rpc.polkachu.com",
}: GetBalanceOnUnion): Promise<bigint> {
  try {
    try {
      const { balances } = await fetcher<{
        balances: Array<{ amount: string; denom: string }>;
      }>(
        `${
          process.env.UNION_REST_URL || "https://union-testnet-api.polkachu.com"
        }/cosmos/bank/v1beta1/balances/${address}`,
        {
          mode: "cors",
          headers: {
            // no-cors
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
          },
        }
      );
      const balance = balances.find(({ denom }) => denom === assetId);
      return BigInt(balance?.amount ?? 0);
    } catch (error) {
      console.log("Failed to get balance using StargateClient", error);
      const client = await StargateClient.connect(unionRpcUrl);
      const { amount } = await client.getBalance(address, assetId);
      return BigInt(amount);
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error;
    console.error(errorMessage);
    raise(`Failed to get balance for ${address} on Union: ${errorMessage}`);
  }
}
