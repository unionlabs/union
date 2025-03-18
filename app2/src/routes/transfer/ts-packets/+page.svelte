<script lang="ts">
import { ucs03ZkgmAbi } from "$lib/abi/ucs03"
import { Option } from "effect"
import { encodeAbiParameters, createPublicClient, http, getContract, parseAbi } from "viem"

const packetComponentAbis = Option.fromNullable(
  ucs03ZkgmAbi.find(a => "name" in a && a.name === "ensureExported")
).pipe(Option.map(a => a.inputs))

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const fungibleAssetOrderAbi = packetComponentAbis.pipe(
  Option.flatMap(a =>
    Option.fromNullable(a.find(as => as.internalType === "struct FungibleAssetOrder"))
  ),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const instructionAbi = packetComponentAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct Instruction"))),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const zkgmPacketAbi = packetComponentAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct ZkgmPacket"))),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

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
</script>

<div class="font-mono break-words">{JSON.stringify(assetOrder, null, 2)}</div>


