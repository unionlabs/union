<script lang="ts">
import { ucs03ZkgmAbi } from "$lib/abi/ucs03"
import { generateSalt } from "$lib/services/shared"
import { Effect, Option, pipe } from "effect"
import { encodeAbiParameters, createPublicClient, http, getContract, parseAbi } from "viem"
import { mainnet } from "viem/chains"

const packetAbis = Option.fromNullable(
  ucs03ZkgmAbi.find(a => "name" in a && a.name === "ensureExported")
).pipe(Option.map(a => a.inputs))

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const fungibleAssetOrderAbi = packetAbis.pipe(
  Option.flatMap(a =>
    Option.fromNullable(a.find(as => as.internalType === "struct FungibleAssetOrder"))
  ),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const instructionAbi = packetAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct Instruction"))),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

// biome-ignore lint/style/noNonNullAssertion: we know this will be included bc the abi is const
const zkgmPacketAbi = packetAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct ZkgmPacket"))),
  Option.map(a => a.components),
  Option.getOrUndefined
)!

const zkgmInstruction = Effect.gen(function* () {
  const salt = yield* generateSalt

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
  // const instruction = encodeAbiParameters(instructionAbi, [1, 3, assetOrder])

  return assetOrder
})

const pt = Effect.runSync(zkgmInstruction)
</script>

<div class="font-mono break-words">{JSON.stringify(pt, null, 2)}</div>


