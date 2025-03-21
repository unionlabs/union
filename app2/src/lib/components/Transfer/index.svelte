<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import {
  hasFailedExit as hasCosmosFailedExit,
  isComplete as isCosmosComplete
} from "$lib/services/transfer-ucs03-cosmos"
import {
  hasFailedExit as hasEvmFailedExit,
  isComplete as isEvmComplete
} from "$lib/services/transfer-ucs03-evm"
import {
  hasFailedExit as hasAptosFailedExit,
  isComplete as isAptosComplete
} from "$lib/services/transfer-ucs03-aptos"
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
import type { TransferStateUnion } from "$lib/components/Transfer/validation.ts"
import { Effect, Option } from "effect"
import { wallets } from "$lib/stores/wallets.svelte"
import TransferAsset from "./ChainAsset/TransferAsset.svelte"
import { WETH_DENOMS } from "$lib/constants/weth-denoms.ts"
import type { Instruction } from "@unionlabs/sdk/ucs03"
import { runPromiseExit } from "effect/Runtime"
import { createEvmToCosmosFungibleAssetOrder, Batch } from "@unionlabs/sdk/ucs03"
import {
  createViemPublicClient,
  ViemPublicClient,
  ViemPublicClientSource,
  readErc20Allowance
} from "@unionlabs/sdk/evm"

import {
  CosmWasmClientDestination,
  createCosmWasmClient,
  CosmosChannelDestination
} from "@unionlabs/sdk/cosmos"
import { sepolia } from "viem/chains"
import { http } from "viem"
import { tapBoth } from "effect/STM"

function getStatus(
  state: TransferStateUnion
): "empty" | "filling" | "processing" | "failed" | "complete" {
  switch (state._tag) {
    case "Empty":
      return "empty"
    case "Evm": {
      if (state.state._tag === "Filling") return "filling"
      if (hasEvmFailedExit(state.state)) return "failed"
      if (isEvmComplete(state.state)) return "complete"
      return "processing"
    }
    case "Cosmos": {
      if (state.state._tag === "Filling") return "filling"
      if (hasCosmosFailedExit(state.state)) return "failed"
      if (isCosmosComplete(state.state)) return "complete"
      return "processing"
    }
    case "Aptos": {
      if (state.state._tag === "Filling") return "filling"
      if (hasAptosFailedExit(state.state)) return "failed"
      if (isAptosComplete(state.state)) return "complete"
      return "processing"
    }
    default:
      return "empty"
  }
}

// Simplified step name extractor
function getStepName(state: TransferStateUnion): string | null {
  switch (state._tag) {
    case "Empty":
      return null
    case "Evm":
      return state.state._tag
    case "Aptos":
      return state.state._tag
    case "Cosmos":
      return state.state._tag
    default:
      return null
  }
}

let isButtonEnabled = $derived(
  getStatus(transfer.state) === "filling" ||
    getStatus(transfer.state) === "failed" ||
    getStatus(transfer.state) === "complete"
)

let buttonText = $derived(
  {
    empty: "Select",
    filling: "Submit",
    processing: "Submitting...",
    failed: "Retry",
    complete: "Submit"
  }[getStatus(transfer.state)]
)

let transferIntents = $derived.by(() => {
  if (transfer.validation._tag !== "Success") return Option.none()
  if (Option.isNone(wallets.evmAddress)) return Option.none()

  const transferValue = transfer.validation.value

  const wethDenom =
    transferValue.sourceChain.universal_chain_id in WETH_DENOMS
      ? Option.some(WETH_DENOMS[transferValue.sourceChain.universal_chain_id])
      : Option.none()

  if (Option.isNone(wethDenom)) return Option.none()
  return Option.some([
    {
      sender: wallets.evmAddress.value,
      receiver: transferValue.receiver,
      baseToken: transferValue.baseToken,
      baseAmount: transferValue.baseAmount,
      quoteAmount: transferValue.baseAmount
    },
    {
      sender: wallets.evmAddress.value,
      receiver: transferValue.receiver,
      baseToken: wethDenom.value,
      baseAmount: 500n,
      quoteAmount: 0n
    }
  ])
})

let instruction: Option.Option<Instruction> = $state(Option.none())
let allowances: Option.Option<Array<{ token: string; allowance: bigint }>> = $state(Option.none())

$effect(() => {
  if (Option.isNone(transferIntents)) return

  intentsToBatch(transferIntents).pipe(
    Effect.tap(batch => (instruction = batch)),
    Effect.runPromiseExit
  )

  checkAllowances(transferIntents).pipe(
    Effect.tap(result => (allowances = result)),
    Effect.runPromiseExit
  )
})

const intentsToBatch = (ti: typeof transferIntents) =>
  Effect.gen(function* () {
    if (Option.isNone(ti)) return Option.none()

    const publicClientSource = yield* createViemPublicClient({
      chain: sepolia, // todo
      transport: http()
    })

    const cosmwasmClientDestination = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    )

    const batch = yield* Effect.gen(function* () {
      const t1 = yield* createEvmToCosmosFungibleAssetOrder(ti.value[0])
      const t2 = yield* createEvmToCosmosFungibleAssetOrder(ti.value[1])
      return Batch([t1, t2])
    }).pipe(
      Effect.provideService(ViemPublicClientSource, { client: publicClientSource }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmwasmClientDestination }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1
      })
    )

    return Option.some(batch)
  })

const checkAllowances = (ti: typeof transferIntents) =>
  Effect.gen(function* () {
    if (Option.isNone(ti)) return Option.none()
    if (Option.isNone(wallets.evmAddress)) return Option.none()

    const publicClientSource = yield* createViemPublicClient({
      chain: sepolia, // todo
      transport: http()
    })

    // Get unique token addresses from the transfer intents
    const tokenAddresses = [...new Set(ti.value.map(intent => intent.baseToken))]

    // The UCS03 contract address that needs the allowance
    const spenderAddress = "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962" // Replace with actual UCS03 contract address

    // Check allowance for each token
    const allowanceChecks = yield* Effect.all(
      tokenAddresses.map(tokenAddress =>
        Effect.gen(function* () {
          const allowance = yield* readErc20Allowance(
            tokenAddress,
            wallets.evmAddress.value,
            spenderAddress
          )
          return { token: tokenAddress, allowance }
        }).pipe(Effect.provideService(ViemPublicClient, { client: publicClientSource }))
      )
    )

    return Option.some(allowanceChecks)
  })
</script>

<Card class="max-w-md relative flex flex-col gap-2">
  <ChainAsset type="source"/>
  <ChainAsset type="destination"/>
  <Amount/>
  <Receiver/>
  <ShowData/>
  <Button
          class="mt-2"
          variant="primary"
          onclick={transfer.submit}
          disabled={!isButtonEnabled || transfer.validation._tag !== "Success"}
  >
    {buttonText}
  </Button>
</Card>



<h2>transfer intents</h2>
<pre>{JSON.stringify(transferIntents,null,2)}</pre>

<h2>instruction</h2>
<pre>{JSON.stringify(instruction,null,2)}</pre>

<h2>allowances</h2>
<pre>{JSON.stringify(allowances,null,2)}</pre>


{#if transfer.state._tag !== "Empty"}
  {#if getStatus(transfer.state) === "filling"}
    <div>Select assets and amounts to begin transfer.</div>
  {:else if getStatus(transfer.state) === "processing"}
    <div>Processing {getStepName(transfer.state) ?? "step"}...</div>
  {:else if getStatus(transfer.state) === "complete"}
    <div style="color: green;">Transfer completed successfully!</div>
  {/if}
  <pre>{JSON.stringify(transfer.state, null, 2)}</pre>
{/if}
