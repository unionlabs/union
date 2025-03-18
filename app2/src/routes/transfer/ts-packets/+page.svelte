<script lang="ts">
import { encodeAbiParameters } from "viem"
import { batchAbi, fungibleAssetOrderAbi } from "@unionlabs/sdk/evm/abi"

const assetOrder = encodeAbiParameters(fungibleAssetOrderAbi, [
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

const batch = encodeAbiParameters(batchAbi, [
  [
    { version: 1, opcode: 3, operand: assetOrder },
    { version: 1, opcode: 3, operand: assetOrder }
  ]
])
</script>

<div class="font-mono break-words">{JSON.stringify(assetOrder, null, 2)}</div>

<div class="font-mono break-words">{JSON.stringify(batch, null, 2)}</div>


