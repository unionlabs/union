import { Effect } from "effect"
import { queryContract } from "./contract.js"
import { AptosPublicClientDestination } from "./client.js"
import { AptosChannelDestination } from "./channel.js"
import { MoveVector } from "@aptos-labs/ts-sdk"

export type Hex = `0x${string}`

export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function* () {
    yield* Effect.log(`Predicting quote token for base token: ${baseToken}`)
    const client = (yield* AptosPublicClientDestination).client
    const config = yield* AptosChannelDestination
    yield* Effect.log("AFTER Fetching client and config:")

    const contract_address = config.ucs03address
    const module_name = "ibc_app"
    const function_name = "predict_wrapped_token"
    const converted_base_token = MoveVector.U8(baseToken)
    const function_arguments = [0, config.channelId.toString(), converted_base_token]

    yield* Effect.log("Predicting quote token for base token:", baseToken)

    const result = yield* queryContract(
      client,
      contract_address,
      module_name,
      function_name,
      [],
      function_arguments
    )

    const wrapped_token = result[0] as Hex

    return wrapped_token
  })
