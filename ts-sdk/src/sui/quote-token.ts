import { Effect } from "effect"
import { SuiChannelDestination } from "./channel.js"
import { SuiPublicClientDestination } from "./client.js"
import { readContract } from "./contract.js"
import { Transaction } from "@mysten/sui/transactions"

export type Hex = `0x${string}`

// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex;
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16));
}


export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function*() {
    const client = (yield* SuiPublicClientDestination).client
    const config = yield* SuiChannelDestination
    yield* Effect.log(`Predicting quote token for base token: ${baseToken} at channel: ${config.channelId} on ZKGM Address: ${config.ucs03address}`)

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "compute_salt"
    const converted_base_token = baseToken

    const tx = new Transaction()
    const function_arguments = [
      tx.pure.u256(0), 
      tx.pure.u32(config.channelId.toString()), 
      tx.pure('vector<u8>', hexToBytes(converted_base_token))
    ]


    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779", // TODO: 
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx
    )
    const rawBytes = result[0].returnValues[0] as number[]; // extract the vector<u8>
    const wrapped_token = rawBytes[0] as Hex

    return wrapped_token
  })

  /*
  import { bcs } from '@mysten/sui/bcs';
 
tx.moveCall({
	target: '0x2::foo::bar',
	arguments: [
		// using vector and option methods
		tx.pure.vector('u8', [1, 2, 3]),
		tx.pure.option('u8', 1),
		tx.pure.option('u8', null),
 
		// Using pure with type arguments
		tx.pure('vector<u8>', [1, 2, 3]),
		tx.pure('option<u8>', 1),
		tx.pure('option<u8>', null),
		tx.pure('vector<option<u8>>', [1, null, 2]),
 
		// Using bcs.serialize
		tx.pure(bcs.vector(bcs.U8).serialize([1, 2, 3])),
		tx.pure(bcs.option(bcs.U8).serialize(1)),
		tx.pure(bcs.option(bcs.U8).serialize(null)),
		tx.pure(bcs.vector(bcs.option(bcs.U8)).serialize([1, null, 2])),
	],
});
*/