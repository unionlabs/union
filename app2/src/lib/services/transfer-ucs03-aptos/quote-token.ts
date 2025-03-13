import { Effect } from "effect"
import { type Address, createPublicClient, fromHex, http } from "viem"
import type { Hex } from "viem"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import type { Channel } from "$lib/schema/channel.ts"
import { request } from "graphql-request"
import { GRAQPHQL_URL } from "@unionlabs/client"
import type { Chain } from "$lib/schema/chain.ts"
import { getChainFromWagmi } from "$lib/wallet/evm"
import { getCosmWasmClient } from "$lib/services/cosmos/clients"
import { tokenWrappingQuery } from "$lib/queries/tokens.svelte.ts"
import { GetQuoteError } from "$lib/services/transfer-ucs03-evm/errors.ts"
import { Aptos, AptosConfig, Network, Deserializer, MoveVector } from "@aptos-labs/ts-sdk"

export const getQuoteToken = (
  sourceChain: Chain,
  base_token: Hex,
  channel: Channel,
  destinationChain: Chain
) =>
  Effect.gen(function* () {
    const { v1_ibc_union_tokens } = yield* Effect.tryPromise({
      try: () =>
        request(GRAQPHQL_URL, tokenWrappingQuery, {
          base_token,
          destination_channel_id: channel.source_channel_id,
          source_chain_id: sourceChain.chain_id
        }),
      catch: error => {
        return new GetQuoteError({ cause: `Failed to get quote token from GraphQL: ${error}` })
      }
    })

    const quote_token = v1_ibc_union_tokens[0]?.wrapping[0]?.unwrapped_address_hex
    if (quote_token) {
      return { type: "UNWRAPPED" as const, quote_token }
    }

    if (destinationChain.rpc_type === "cosmos") {
      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      const client = yield* getCosmWasmClient(rpc.toString())
      const predictedQuoteToken = yield* Effect.tryPromise({
        try: () =>
          client.queryContractSmart(fromHex(channel.destination_port_id, "string"), {
            predict_wrapped_token: {
              path: "0",
              channel: channel.destination_channel_id,
              token: base_token
            }
          }),
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (Cosmos): ${error}` })
      }).pipe(Effect.map(res => res.wrapped_token as Hex))

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "evm") {
      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      const client = createPublicClient({
        chain: getChainFromWagmi(Number.parseInt(channel.destination_chain_id)),
        transport: http(rpc.toString())
      })
      const predictedQuoteToken = yield* Effect.tryPromise({
        try: () =>
          client.readContract({
            address: `0x${channel.destination_port_id}`,
            abi: ucs03ZkgmAbi,
            functionName: "predictWrappedToken",
            args: [0, channel.destination_channel_id, base_token]
          }) as Promise<[Address, string]>,
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (EVM): ${error}` })
      }).pipe(Effect.map(([address]) => address))

      return { type: "NEW_WRAPPED" as const, quote_token: predictedQuoteToken }
    }

    if (destinationChain.rpc_type === "aptos") {
      let network: Network;
      let rpcUrl: string;

      const rpc = yield* destinationChain
        .requireRpcUrl("rpc")
        .pipe(Effect.mapError(err => new GetQuoteError({ cause: err.message })))

      console.info("rpc: ", rpc.origin)
      if (channel.destination_chain_id === "250") {
        network = Network.TESTNET;
        rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1";
      } else {
        return yield* Effect.fail(new GetQuoteError({ cause: `Unsupported Aptos network: ${channel.destination_chain_id}` }));
      }
    
      // const config = new AptosConfig({ network, fullnode: rpc.origin+"/v1" }); //TODO: rpc.origin is coming without "/v1" at the end, discuss this later
      //And also this rpc.origin returns:
      // :5173/transfer?source=union-testnet-9&destination=250&asset=0x6d756e6f:1 Access to XMLHttpRequest at 'https://rpc.250.movement.chain.kitchen/v1/accounts/0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84/module/ibc_app' from origin 'http://localhost:5173' has been blocked by CORS policy: Request header field x-aptos-typescript-sdk-origin-method is not allowed by Access-Control-Allow-Headers in preflight response.
      const config = new AptosConfig({ network, fullnode: rpcUrl });
      const aptosClient = new Aptos(config);
      
      
      const output = yield* Effect.tryPromise({
        try: () =>
          aptosClient.view({
            payload: {
              function: `${channel.destination_port_id}::ibc_app::predict_wrapped_token`,
              typeArguments: [],
              functionArguments: [
                0, // path
                channel.destination_channel_id,
                MoveVector.U8(base_token)              
            ]
            }
          }),
        catch: error =>
          new GetQuoteError({ cause: `Failed to predict quote token (Aptos): ${error}` })
      });
    
      const wrappedAddressHex = output[0]?.toString();
      if (!wrappedAddressHex) {
        return yield* Effect.fail(new GetQuoteError({ cause: "Failed to get wrapped address from Aptos" }));
      }
      return { type: "NEW_WRAPPED" as const, quote_token: wrappedAddressHex };
    }
    

    return yield* Effect.fail(
      new GetQuoteError({ cause: `${destinationChain.rpc_type} not supported` })
    )
  })
