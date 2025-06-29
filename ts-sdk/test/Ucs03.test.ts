import { UCS03 } from "@unionlabs/sdk"
import { Address } from "@unionlabs/sdk/schema"
import * as Address from "@unionlabs/sdk/schema/Address"
import { createEvmToCosmosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
type A = 0

const b = Address.Bech32

const fwd = UCS03.Abi.Abi
type C = UCS03.Abi.A

declare const instr: Ucs03Instruction.Instruction

const encoded = Ucs03Instruction.encodeAbi(instr)

const order = createEvmToCosmosFungibleAssetOrder({
  sender: AddressEvmZkgm.make("0x00"),
  receiver: "0x123",
  quoteAmount: 0n,
})

const i = Ucs03Instruction.Batch.make({
  operand: [
    Ucs03Instruction.FungibleAssetOrder.fromOperand(order),
    Ucs03Instruction.Forward({}),
  ],
})
