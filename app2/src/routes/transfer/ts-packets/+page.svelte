<script lang="ts">
import { encodeAbiParameters } from "viem"
import { batchAbi, fungibleAssetOrderAbi } from "@unionlabs/sdk/evm/abi"

const fungibleAssetOrder = (
  operand: Parameters<typeof encodeAbiParameters<typeof fungibleAssetOrderAbi>>[1]
) => ({
  opcode: 3,
  version: 1,
  operand: encodeAbiParameters(fungibleAssetOrderAbi, operand)
})

const batch = (
  instructions: Array<{ version: number; opcode: number; operand: `0x${string}` }>
) => ({
  opcode: 2,
  version: 0,
  operand: encodeAbiParameters(batchAbi, [instructions])
})

const exampleOrder = fungibleAssetOrder([
  "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
  "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
  "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
  4n,
  "muno",
  "muno",
  18,
  0n,
  "0x74d5b8eacfeb0dadaaf66403f40e304b3ef968b3",
  4n
])

const exampleBatch = batch([exampleOrder, exampleOrder])
</script>

<div class="font-mono break-words">{JSON.stringify(exampleOrder, null, 2)}</div>

<div class="font-mono break-words">{JSON.stringify(exampleBatch, null, 2)}</div>


