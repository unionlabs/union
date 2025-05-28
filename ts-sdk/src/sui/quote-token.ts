import { Effect } from "effect"
import { SuiChannelDestination } from "./channel.js"
import { SuiPublicClientDestination } from "./client.js"
import { readContract } from "./contract.js"
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { Transaction } from "@mysten/sui/transactions"

export type Hex = `0x${string}`

// turn a hex string like "0xdeadbeef" into a number[] of bytes
function hexToBytes(hex: string): number[] {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex;
  return h.match(/.{1,2}/g)!.map(b => parseInt(b, 16));
}

function bytesToHex(bs: number[]): `0x${string}` {
  return "0x" + bs.map(b => b.toString(16).padStart(2, "0")).join("");
}

export const predictQuoteToken = (baseToken: Hex) =>
  Effect.gen(function*() {
    yield* Effect.log(`Predicting quote token for base token: ${baseToken}`)
    const client = (yield* SuiPublicClientDestination).client
    const config = yield* SuiChannelDestination
    yield* Effect.log("AFTER Fetching client and config:")

    const contract_address = config.ucs03address
    const module_id = "zkgm_relay"
    const function_name = "compute_salt"
    const converted_base_token = baseToken
    const keypair = new Ed25519Keypair();

    const tx = new Transaction()
    const function_arguments = [
      tx.pure.u256(0), 
      tx.pure.u32(config.channelId.toString()), 
      tx.pure('vector<u8>', hexToBytes(converted_base_token))
    ]

    yield* Effect.log("Predicting quote token for base token:", baseToken)

    const result = yield* readContract(
      client,
      "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
      contract_address,
      module_id,
      function_name,
      [],
      function_arguments,
      tx
    )


    console.info("Result from readContract:", result[0].returnValues[0] as string)
    const wrapped_token = result[0] as Hex

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