import { Effect } from "effect";
import { createPublicClient, http } from "viem";
import { type Hex } from "viem";
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts";
import { getChainFromWagmi } from "$lib/wallet/evm";
import type { Channel } from "$lib/schema/channel.ts";
import { request } from "graphql-request";
import { GRAQPHQL_URL } from "@unionlabs/client";
import { graphql } from "gql.tada";

//Quick copy, needs to use what we build

type ChainType = "evm" | "cosmos" | "aptos";

const tokenWrappingQuery = graphql(/* GraphQL */ `
    query QueryTokenWrapping($source_chain_id: String!, $base_token: String!, $destination_channel_id: Int!) {
        v1_ibc_union_tokens(where: {_and: {chain: {chain_id: {_eq: $source_chain_id}}, denom: {_eq: $base_token}, wrapping: {_and: {index: {_eq: 0}, destination_channel_id: {_eq: $destination_channel_id}}}}}) {
            wrapping {
                unwrapped_address_hex
            }
        }
    }
`);

interface GraphQLResponse {
  v1_ibc_union_tokens: { wrapping: { unwrapped_address_hex: string }[] }[];
}

export const getQuoteToken = (
  source_chain_id: string,
  base_token: Hex,
  channel: Channel,
  chainType: ChainType
) => Effect.gen(function* () {
  // Check if token is wrapped
  const { v1_ibc_union_tokens } = yield* Effect.tryPromise({
    try: () => request<GraphQLResponse>(GRAQPHQL_URL, tokenWrappingQuery, {
      base_token,
      destination_channel_id: parseInt(channel.source_channel_id),
      source_chain_id
    }),
    catch: (error) => new Error("GraphQL fetch failed", { cause: error })
  });

  const quote_token = v1_ibc_union_tokens[0]?.wrapping[0]?.unwrapped_address_hex;
  if (quote_token) {
    return { type: "UNWRAPPED" as const, quote_token };
  }

  // Handle chain-specific prediction
  if (chainType === "evm") {
    const client = createPublicClient({
      chain: getChainFromWagmi(parseInt(channel.destination_chain_id)),
      transport: http("https://rpc.testnet-9.union.build")
    });

    const predictedQuoteToken = yield* Effect.tryPromise({
      try: () => client.readContract({
        address: `0x${channel.destination_port_id}` as const,
        abi: ucs03ZkgmAbi,
        functionName: "predictWrappedToken" as const,
        args: [0, parseInt(channel.destination_channel_id), base_token] as const
      }) as Promise<[Hex, string]>,
      catch: (error) => new Error("EVM contract call failed", { cause: error })
    }).pipe(
      Effect.map(([address]) => address)
    );

    return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken };
  }

  return yield* Effect.fail(new Error(`${chainType} not supported`));
});

export const runthis = async (transfer: {
  sourceChain?: { chain_id: string; rpc_type: string };
  baseToken?: { denom: string };
  channel?: Channel;
}) => {
  if (!transfer.sourceChain?.chain_id || !transfer.baseToken?.denom || !transfer.channel || !transfer.sourceChain?.rpc_type) {
    console.log("Missing parameters");
    return;
  }

  const res = await Effect.runPromise(
    getQuoteToken(
      transfer.sourceChain.chain_id,
      transfer.baseToken.denom as Hex,
      transfer.channel,
      transfer.sourceChain.rpc_type as ChainType
    )
  );
  console.log("Result:", res);
};