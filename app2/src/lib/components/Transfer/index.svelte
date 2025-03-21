<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { LockedTransfer } from "./locked-transfer"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
import { lockedTransferStore } from "./locked-transfer.svelte.ts"
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
import { WETH_DENOMS } from "$lib/constants/weth-denoms.ts"
import { createEvmToCosmosFungibleAssetOrder, Instruction } from "@unionlabs/sdk/ucs03"
import {
  createViemPublicClient,
  ViemPublicClient,
  ViemPublicClientSource,
  readErc20Allowance
} from "@unionlabs/sdk/evm"
import { Data } from "effect"

import {
  CosmWasmClientDestination,
  createCosmWasmClient,
  CosmosChannelDestination
} from "@unionlabs/sdk/cosmos"
import { sepolia } from "viem/chains"
import { http } from "viem"
import { truncate } from "$lib/utils/format.ts"
import { type TransferStep, Filling, ApprovalRequired, SubmitInstruction } from "./transfer-step"

function getStatus(
  state: TransferStateUnion
): "empty" | "filling" | "processing" | "failed" | "complete" {
  if (state._tag === "Empty") {
    return "empty"
  }

  if (state._tag === "Evm") {
    if (state.state._tag === "Filling") return "filling"
    if (hasEvmFailedExit(state.state)) return "failed"
    if (isEvmComplete(state.state)) return "complete"
    return "processing"
  }

  if (state._tag === "Cosmos") {
    if (state.state._tag === "Filling") return "filling"
    if (hasCosmosFailedExit(state.state)) return "failed"
    if (isCosmosComplete(state.state)) return "complete"
    return "processing"
  }

  if (state._tag === "Aptos") {
    if (state.state._tag === "Filling") return "filling"
    if (hasAptosFailedExit(state.state)) return "failed"
    if (isAptosComplete(state.state)) return "complete"
    return "processing"
  }

  return "empty"
}

// Simplified step name extractor
function getStepName(state: TransferStateUnion): string | null {
  if (state._tag === "Empty") {
    return null
  }

  if (state._tag === "Evm") {
    return state.state._tag
  }

  if (state._tag === "Aptos") {
    return state.state._tag
  }

  if (state._tag === "Cosmos") {
    return state.state._tag
  }

  return null
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
let requiredApprovals = $derived.by(() => {
  if (Option.isNone(transferIntents) || Option.isNone(allowances)) return Option.none()

  // Create a map of token to required amount from transfer intents
  const requiredAmounts = new Map<string, bigint>()
  for (const intent of transferIntents.value) {
    const currentAmount = requiredAmounts.get(intent.baseToken) || 0n
    requiredAmounts.set(intent.baseToken, intent.baseAmount)
  }

  // Filter for tokens that need approval (allowance < required amount)
  const tokensNeedingApproval = allowances.value
    .filter(({ token, allowance }) => {
      const requiredAmount = requiredAmounts.get(token) || 0n
      return allowance < requiredAmount
    })
    .map(({ token }) => ({
      token,
      requiredAmount: requiredAmounts.get(token) || 0n
    }))

  return tokensNeedingApproval.length > 0 ? Option.some(tokensNeedingApproval) : Option.none()
})

// Derive the steps based on required approvals and instruction
let transferSteps = $derived.by(() => {
  const steps: Array<TransferStep> = [Filling()]

  // Add approval steps if needed
  if (Option.isSome(requiredApprovals)) {
    // Find the allowance for each token that needs approval
    for (const approval of requiredApprovals.value) {
      if (Option.isSome(allowances)) {
        const tokenAllowance = allowances.value.find(a => a.token === approval.token)
        if (tokenAllowance) {
          steps.push(
            ApprovalRequired({
              token: approval.token,
              requiredAmount: approval.requiredAmount,
              currentAllowance: tokenAllowance.allowance
            })
          )
        }
      }
    }
  }

  // Add the instruction submission step if we have an instruction
  if (Option.isSome(instruction)) {
    steps.push(SubmitInstruction({ instruction: instruction.value }))
  }

  return steps.length > 0 ? Option.some(steps) : Option.none()
})

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
      return new Instruction.Batch({ operand: [t1, t2] })
    }).pipe(
      Effect.provideService(ViemPublicClientSource, {
        client: publicClientSource
      }),
      Effect.provideService(CosmWasmClientDestination, {
        client: cosmwasmClientDestination
      }),
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
        }).pipe(
          Effect.provideService(ViemPublicClient, {
            client: publicClientSource
          })
        )
      )
    )

    return Option.some(allowanceChecks)
  })

let showDetails = $state(false)
let currentPage = $state(0)

function goToNextPage() {
  if (Option.isSome(transferSteps) && currentPage < transferSteps.value.length - 1) {
    currentPage++
  }
}

function goToPreviousPage() {
  if (currentPage > 0) {
    currentPage--
    
    // If we're going back to the filling page (page 0), unlock the transfer
    if (currentPage === 0) {
      lockedTransferStore.unlock()
    }
  }
}

// Determine which button text to show based on current page and state
let actionButtonText = $derived.by(() => {
  if (Option.isNone(transferSteps)) return "Submit"

  const currentStep = transferSteps.value[currentPage]

  if (currentPage === transferSteps.value.length - 1) {
    return "Complete"
  }

  if (currentStep._tag === "Filling") {
    return "Continue"
  }

  if (currentStep._tag === "ApprovalRequired") {
    return "Approve"
  }

  if (currentStep._tag === "SubmitInstruction") {
    return "Submit"
  }

  return "Next"
})

// Handle the action button click based on current page
function handleActionButtonClick() {
  if (Option.isNone(transferSteps)) return

  const currentStep = transferSteps.value[currentPage]

  if (currentStep._tag === "Filling") {
    // Lock the transfer values before proceeding
    if (!lockedTransferStore.isLocked()) {
      const newLockedTransfer = LockedTransfer.fromTransfer(
        transfer.sourceChain,
        transfer.destinationChain,
        transfer.channel,
        transferSteps
      )

      // If we couldn't create a locked transfer, don't proceed
      if (Option.isNone(newLockedTransfer)) {
        console.error("Failed to lock transfer values")
        return
      }
      
      lockedTransferStore.lock(newLockedTransfer.value)
    }
    goToNextPage()
    return
  }

  if (currentStep._tag === "ApprovalRequired") {
    // Here you would handle the approval action
    // For now, just go to next page
    goToNextPage()
    return
  }

  if (currentStep._tag === "SubmitInstruction") {
    // Here you would handle the submit action
    transfer.submit()
    return
  }
}
</script>

<Card divided class="w-sm my-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden">
  <div class="p-4 w-full">
    <StepProgressBar 
      class="w-full"
      currentStep={currentPage + 1} 
      totalSteps={lockedTransferStore.isLocked() 
        ? lockedTransferStore.get().value.steps.length 
        : transferSteps.pipe(Option.map(ts => ts.length), Option.getOrElse(() => 1))}
      stepDescriptions={lockedTransferStore.isLocked()
        ? lockedTransferStore.get().value.steps.map(step => {
            if (step._tag === "Filling") {
              return "Configure your transfer details"
            }
            if (step._tag === "ApprovalRequired") {
              return "Approve token spending"
            }
            if (step._tag === "SubmitInstruction") {
              return "Submit transfer to blockchain"
            }
            return "Transfer step"
          })
        : Option.isSome(transferSteps) 
          ? transferSteps.value.map(step => {
              if (step._tag === "Filling") {
                return "Configure your transfer details"
              }
              if (step._tag === "ApprovalRequired") {
                return "Approve token spending"
              }
              if (step._tag === "SubmitInstruction") {
                return "Submit transfer to blockchain"
              }
              return "Transfer step"
            })
          : ["Configure your transfer"]}
    />
  </div>
  
  <!-- Sliding pages container -->
  <div class="relative flex-1 overflow-hidden">
    <!-- Pages wrapper with horizontal sliding -->
    <div 
      class="absolute inset-0 flex transition-transform duration-300 ease-in-out"
      style="transform: translateX(-{currentPage * 100}%);"
    >
      <!-- Page 1: Filling -->
      <FillingPage 
        onContinue={handleActionButtonClick}
        actionButtonText={actionButtonText}
      />

      <!-- Dynamic pages for each step -->
      {#if lockedTransferStore.isLocked()}
        {#each lockedTransferStore.get().value.steps.slice(1) as step, i}
          {#if step._tag === "ApprovalRequired"}
            <ApprovalPage
              stepIndex={i + 1}
              onBack={goToPreviousPage}
              onApprove={handleActionButtonClick}
              actionButtonText={actionButtonText}
            />
          {:else if step._tag === "SubmitInstruction"}
            <SubmitPage
              stepIndex={i + 1}
              onBack={goToPreviousPage}
              onSubmit={handleActionButtonClick}
              actionButtonText={actionButtonText}
            />
          {/if}
        {/each}
      {/if}
    </div>
  </div>
  
  {#if showDetails}
    <ShowData />
  {/if}
</Card>


<!-- Debug info can be hidden in production -->
{#if lockedTransferStore.isLocked() || Option.isSome(transferSteps)}
  <div class="mt-4">
    <h3 class="text-lg font-semibold">Current Page: {currentPage + 1}/{lockedTransferStore.isLocked() ? lockedTransferStore.get().value.steps.length : Option.isSome(transferSteps) ? transferSteps.value.length : 0}</h3>
    <h4 class="text-md">Steps to complete transfer:</h4>
    <ol class="list-decimal pl-5 mt-2">
      {#each lockedTransferStore.isLocked() ? lockedTransferStore.get().value.steps : Option.isSome(transferSteps) ? transferSteps.value : [] as step, index}
        <li class="mb-2" class:font-bold={index === currentPage}>
          {#if step._tag === "Filling"}
            <div>Configure transfer details</div>
          {:else if step._tag === "ApprovalRequired"}
            <div>
              Approve token: <span class="font-mono">{truncate(step.token, 8, "middle")}</span>
              <div class="text-sm">
                Current allowance: {step.currentAllowance.toString()}
                <br />
                Required amount: {step.requiredAmount.toString()}
              </div>
            </div>
          {:else if step._tag === "SubmitInstruction"}
            <div>Submit transfer instruction</div>
          {/if}
        </li>
      {/each}
    </ol>
  </div>
{/if}

<h2>transfer intents</h2>
<pre>{JSON.stringify(transferIntents, null, 2)}</pre>

<h2>instruction</h2>
<pre>{JSON.stringify(instruction, null, 2)}</pre>

<h2>allowances</h2>
<pre>{JSON.stringify(allowances, null, 2)}</pre>

<h2>required approvals</h2>
<pre>{JSON.stringify(requiredApprovals, null, 2)}</pre>

<h2>transfer steps</h2>
<pre>{JSON.stringify(transferSteps, null, 2)}</pre>

<h2>locked transfer</h2>
<pre>{JSON.stringify(lockedTransferStore.get(), null, 2)}</pre>

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
