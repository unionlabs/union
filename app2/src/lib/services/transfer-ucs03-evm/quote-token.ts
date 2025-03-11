import {Effect} from "effect";
import {createPublicClient, fromHex, http} from "viem";
import {type Hex} from "viem";
import {ucs03ZkgmAbi} from "$lib/abi/ucs03.ts";
import type {Channel} from "$lib/schema/channel.ts";
import {request} from "graphql-request";
import {GRAQPHQL_URL} from "@unionlabs/client";
import {graphql} from "gql.tada";
import {Chain, type RpcType} from "$lib/schema/chain.ts";
import {getChainFromWagmi} from "$lib/wallet/evm";
import {getCosmWasmClient} from "$lib/services/cosmos/clients";
import {type CosmosChainId, cosmosRpcs} from "$lib/services/cosmos/rpc.ts";

//quick and drity

const tokenWrappingQuery = graphql(/* GraphQL */ `
    query QueryTokenWrapping($source_chain_id: String!, $base_token: String!, $destination_channel_id: Int!) {
        v1_ibc_union_tokens(where: {_and: {chain: {chain_id: {_eq: $source_chain_id}}, denom: {_eq: $base_token}, wrapping: {_and: {index: {_eq: 0}, destination_channel_id: {_eq: $destination_channel_id}}}}}) {
            wrapping {
                unwrapped_address_hex
            }
        }
    }
`);

export const getQuoteToken = (
  sourceChain: Chain,
  base_token: Hex,
  channel: Channel,
  destinationChain: Chain
) => Effect.gen(function* () {
  const {v1_ibc_union_tokens} = yield* Effect.tryPromise({
    try: () => request(GRAQPHQL_URL, tokenWrappingQuery, {
      base_token,
      destination_channel_id: channel.source_channel_id, // Convert to Int
      source_chain_id: sourceChain.chain_id
    }),
    catch: (error) => {
      console.error("@unionlabs/client-[getQuoteToken]", error);
      return new Error("Failed to get quote token from GraphQL", {cause: error});
    }
  });


  const quote_token = v1_ibc_union_tokens[0]?.wrapping[0]?.unwrapped_address_hex;
  if (quote_token) {
    return {type: "UNWRAPPED" as const, quote_token};
  }


  if (destinationChain.rpc_type === "cosmos") {
    const rpc = cosmosRpcs[channel.destination_chain_id as CosmosChainId]
    const client = yield* getCosmWasmClient(rpc);
    const predictedQuoteToken = yield* Effect.tryPromise({
      try: () => client.queryContractSmart(
        fromHex(`0x${channel.destination_port_id}`, "string"),
        {
          predict_wrapped_token: {
            path: "0",
            channel: channel.destination_channel_id,
            token: base_token
          }
        }
      ),
      catch: (error) => new Error("Failed to predict wrapped token (Cosmos)", {cause: error})
    }).pipe(
      Effect.map(res => res.wrapped_token as Hex)
    );

    return {type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken};
  }

  if (destinationChain.rpc_type === "evm") {
    const rpc = "https://rpc.testnet-9.union.build";
    const client = createPublicClient({
      chain: getChainFromWagmi(parseInt(channel.destination_chain_id)),
      transport: http(rpc)
    });
    const predictedQuoteToken = yield* Effect.tryPromise({
      try: () => client.readContract({
        address: `0x${channel.destination_port_id}`,
        abi: ucs03ZkgmAbi,
        functionName: "predictWrappedToken",
        args: [0, channel.destination_channel_id, base_token]
      }) as Promise<[Hex, string]>,
      catch: (error) => new Error("Failed to predict token (EVM)", {cause: error})
    }).pipe(
      Effect.map(([address]) => address)
    );

    return {type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken};
  }

  return yield* Effect.fail(new Error(`${destinationChain.rpc_type} not supported`));
});
