<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import StepProgressBar from "$lib/components/ui/StepProgressBar.svelte"
import { LockedTransfer } from "./locked-transfer.ts"
import ShowData from "$lib/components/Transfer/ShowData.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import FillingPage from "./pages/FillingPage.svelte"
import ApprovalPage from "./pages/ApprovalPage.svelte"
import SubmitPage from "./pages/SubmitPage.svelte"
import { lockedTransferStore } from "./locked-transfer.svelte.ts"
import { Effect, Match, Option, pipe } from "effect"
import { wallets } from "$lib/stores/wallets.svelte"
import { WETH_DENOMS } from "$lib/constants/weth-denoms.ts"
import {
  createCosmosToCosmosFungibleAssetOrder,
  createCosmosToEvmFungibleAssetOrder,
  createEvmToCosmosFungibleAssetOrder,
  createEvmToEvmFungibleAssetOrder,
  type Instruction
} from "@unionlabs/sdk/ucs03"
import { Batch } from "@unionlabs/sdk/ucs03/instruction.ts"
import {
  createViemPublicClient,
  EvmChannelDestination,
  readErc20Allowance,
  ViemPublicClient,
  ViemPublicClientDestination,
  ViemPublicClientSource
} from "@unionlabs/sdk/evm"

import {
  CosmosChannelDestination,
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient
} from "@unionlabs/sdk/cosmos"
import { fromHex, http, isHex } from "viem"
import { truncate } from "$lib/utils/format.ts"
import {
  ApprovalRequired,
  Filling,
  getStepDescription,
  SubmitInstruction,
  type TransferStep,
  WaitForIndex
} from "./transfer-step.ts"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import IndexPage from "$lib/components/Transfer/pages/IndexPage.svelte"

let showDetails = $state(false)
let currentPage = $state(0)
let instruction: Option.Option<Instruction> = $state(Option.none())
let allowances: Option.Option<Array<{ token: string; allowance: bigint }>> = $state(Option.none())

//This should now handle cosmos, evm and aptos (when aptos is implemented)
let transferIntents = $derived.by(() => {
  if (transfer.validation._tag !== "Success") return Option.none()
  const transferValue = transfer.validation.value

  const sender = wallets.getAddressForChain(transferValue.sourceChain)

  if (Option.isNone(sender)) return Option.none()

  const wethDenom =
    transferValue.sourceChain.universal_chain_id in WETH_DENOMS
      ? Option.some(WETH_DENOMS[transferValue.sourceChain.universal_chain_id])
      : Option.none()

  if (transferValue.sourceChain.rpc_type === "evm") {
    if (Option.isNone(wethDenom)) return Option.none()
    return Option.some([
      {
        sender: sender.value,
        receiver: transferValue.receiver,
        baseToken: transferValue.baseToken,
        baseAmount: transferValue.baseAmount,
        quoteAmount: transferValue.baseAmount
      },
      {
        sender: sender.value,
        receiver: transferValue.receiver,
        baseToken: wethDenom.value,
        baseAmount: 500n,
        quoteAmount: 0n
      }
    ])
  }

  if (transferValue.sourceChain.rpc_type === "cosmos") {
    return Option.some([
      {
        sender: sender.value,
        receiver: transferValue.receiver,
        baseToken: isHex(transferValue.baseToken)
          ? fromHex(transferValue.baseToken, "string")
          : transferValue.baseToken,
        baseAmount: transferValue.baseAmount,
        quoteAmount: transferValue.baseAmount
      }
    ])
  }
})

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
    steps.push(WaitForIndex())
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
    if (
      Option.isNone(ti) ||
      Option.isNone(transfer.sourceChain) ||
      Option.isNone(transfer.channel) ||
      Option.isNone(transfer.destinationChain) ||
      Option.isNone(transfer.ucs03address)
    ) {
      return Option.none()
    }

    const source = transfer.sourceChain.value.rpc_type
    const destination = transfer.destinationChain.value.rpc_type

    const provideViemPublicClientSource = Effect.provideServiceEffect(
      ViemPublicClientSource,
      pipe(
        transfer.sourceChain.value.toViemChain(),
        Option.map(chain =>
          createViemPublicClient({
            chain,
            transport: http()
          })
        ),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const provideViemPublicClientDestination = Effect.provideServiceEffect(
      ViemPublicClientDestination,
      pipe(
        transfer.destinationChain.value.toViemChain(),
        Option.map(chain =>
          createViemPublicClient({
            chain,
            transport: http()
          })
        ),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const provideCosmWasmClientSource = Effect.provideServiceEffect(
      CosmWasmClientSource,
      pipe(
        Option.some("https://rpc.rpc-node.union-testnet-10.union.build"),
        // transfer.sourceChain.value.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const provideCosmWasmClientDestination = Effect.provideServiceEffect(
      CosmWasmClientDestination,
      pipe(
        Option.some("https://rpc.rpc-node.union-testnet-10.union.build"),
        //transfer.destinationChain.value.getRpcUrl("rpc"),
        Option.map(createCosmWasmClient),
        Effect.flatten,
        Effect.map(client => ({ client }))
      )
    )

    const evmChannelDestinationEffect = Effect.succeed({
      ucs03address: transfer.channel.value.destination_port_id,
      channelId: transfer.channel.value.destination_channel_id
    })

    const cosmosChannelDestinationEffect = Effect.succeed({
      ucs03address: fromHex(transfer.channel.value.destination_port_id, "string"),
      channelId: transfer.channel.value.destination_channel_id
    })

    const provideEvmChannelDestination = Effect.provideServiceEffect(
      EvmChannelDestination,
      evmChannelDestinationEffect
    )

    const provideCosmosChannelDestination = Effect.provideServiceEffect(
      CosmosChannelDestination,
      cosmosChannelDestinationEffect
    )

    const batchEffect = Effect.gen(function* () {
      console.log(`batch: Transfer intent value:`, ti.value)

      const orders = yield* Match.value([source, destination]).pipe(
        Match.when(["evm", "cosmos"], () => {
          console.log("batch: Matched EVM -> Cosmos pattern", ti.value)
          return Effect.all([
            Effect.tap(createEvmToCosmosFungibleAssetOrder(ti.value[0]), order =>
              Effect.sync(() => console.log("batch: First order created", order))
            ),
            Effect.tap(createEvmToCosmosFungibleAssetOrder(ti.value[1]), order =>
              Effect.sync(() => console.log("batch: Second order created", order))
            )
          ]).pipe(
            Effect.tap(orders =>
              Effect.sync(() => console.log("batch: All orders created", orders))
            ),
            Effect.catchAll(error => {
              console.error("batch: Error creating orders", error.cause)
              return Effect.fail(error)
            }),
            provideCosmosChannelDestination,
            provideViemPublicClientSource,
            provideCosmWasmClientDestination
          )
        }),
        Match.when(["evm", "evm"], () => {
          console.log("batch: Matched EVM -> EVM pattern")
          return Effect.all([
            createEvmToEvmFungibleAssetOrder(ti.value[0]),
            createEvmToEvmFungibleAssetOrder(ti.value[1])
          ]).pipe(
            Effect.tap(orders =>
              Effect.sync(() => console.log("batch: EVM->EVM orders created", orders))
            ),
            Effect.catchAll(error => {
              console.error("batch: Error creating EVM->EVM orders", error.cause)
              return Effect.fail(error)
            }),
            provideViemPublicClientSource,
            provideViemPublicClientDestination,
            provideEvmChannelDestination
          )
        }),
        Match.when(["cosmos", "evm"], () => {
          console.log("batch: Matched Cosmos -> EVM pattern")
          return createCosmosToEvmFungibleAssetOrder(ti.value[0]).pipe(
            Effect.tap(order =>
              Effect.sync(() => console.log("batch: Cosmos->EVM order created", order))
            ),
            Effect.catchAll(error => {
              console.error("batch: Error creating Cosmos->EVM order", error.cause)
              return Effect.fail(error)
            }),
            provideCosmWasmClientSource,
            provideViemPublicClientDestination,
            provideEvmChannelDestination
          )
        }),
        Match.when(["cosmos", "cosmos"], () => {
          console.log("batch: Matched Cosmos -> Cosmos pattern")
          return createCosmosToCosmosFungibleAssetOrder(ti.value[0]).pipe(
            Effect.tap(order =>
              Effect.sync(() => console.log("batch: Cosmos->Cosmos order created", order))
            ),
            Effect.catchAll(error => {
              console.error("batch: Error creating Cosmos->Cosmos order", error.cause)
              return Effect.fail(error)
            }),
            provideCosmWasmClientSource,
            provideCosmWasmClientDestination,
            provideCosmosChannelDestination
          )
        }),
        Match.orElse(x => {
          console.log(`batch: No match found for ${source} -> ${destination}, throwing error`)
          throw new Error(`Unsupported source/destination combination: ${source} -> ${destination}`)
        })
      )

      // Handle both array and single order cases
      console.log(`batch: Orders created:`, orders)
      const batch = Array.isArray(orders) ? Batch(orders) : Batch([orders])
      console.log(`batch: Batch created:`, batch)
      return batch
    }).pipe(
      // Provide additional services and let Effect handle dependency resolution
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: fromHex(transfer.channel.value.destination_port_id, "string"),
        channelId: transfer.channel.value.destination_channel_id
      }),

      Effect.provideService(EvmChannelDestination, {
        ucs03address: transfer.channel.value.source_port_id,
        channelId: transfer.channel.value.source_channel_id
      })
    )

    const batchResult = yield* batchEffect
    return Option.some(batchResult)
  })

const checkAllowances = (ti: typeof transferIntents) =>
  Effect.gen(function* () {
    console.info("Checking allowances")
    if (Option.isNone(ti)) return Option.none()
    if (Option.isNone(transfer.sourceChain)) return Option.none()
    if (Option.isNone(transfer.ucs03address)) return Option.none()

    const sourceChain = transfer.sourceChain.value
    const chainType = sourceChain.rpc_type
    const spenderAddress = transfer.ucs03address.value

    // Get the sender's address for the source chain.
    const sender = wallets.getAddressForChain(sourceChain)

    if (Option.isNone(sender)) return Option.none()

    // Get unique token addresses from the transfer intents.
    const tokenAddresses = [...new Set(ti.value.map(intent => intent.baseToken))]

    if (chainType === "evm") {
      // For EVM chains use the existing logic.
      const viemChain = sourceChain.toViemChain()
      if (Option.isNone(viemChain)) return Option.none()
      const publicClientSource = yield* createViemPublicClient({
        chain: viemChain.value,
        transport: http()
      })

      const allowanceChecks = yield* Effect.all(
        tokenAddresses.map(tokenAddress =>
          Effect.gen(function* () {
            const allowance = yield* readErc20Allowance(
              tokenAddress,
              sender.value, // EVM sender address
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
    }

    if (chainType === "cosmos") {
      // For Cosmos chains use a CosmWasm client to query CW20 allowances.
      const rpcUrl = sourceChain.getRpcUrl("rpc")
      if (Option.isNone(rpcUrl)) return Option.none()
      const cosmwasmClient = yield* createCosmWasmClient(
        "https://rpc.rpc-node.union-testnet-10.union.build"
      )

      // Query each token (assumed to be a CW20 contract) for the allowance.
      const allowanceChecks = yield* Effect.all(
        tokenAddresses.map(tokenAddress =>
          Effect.gen(function* () {
            const decodedAddr = fromHex(tokenAddress, "string")

            if (!isValidBech32ContractAddress(decodedAddr)) {
              console.log("It's native token, returning none. Token:", tokenAddress)
              return Option.none()
            }

            // TODO:
            // const allowance = yield* readCw20Allowance(contractAddress, walletAddress, spender).pipe(withClient)
            // use it like this when deployed new ts-sdk
            const owner = yield* sourceChain.toCosmosDisplay(sender.value)
            const result = yield* Effect.tryPromise({
              try: () =>
                cosmwasmClient.queryContractSmart(decodedAddr, {
                  allowance: {
                    owner: owner,
                    spender: spenderAddress
                  }
                }),
              catch: e => console.info("Error: ", e)
            })

            return {
              token: tokenAddress,
              allowance: BigInt(result.allowance)
            }
          }).pipe(
            Effect.provideService(CosmWasmClientSource, {
              client: cosmwasmClient
            }),
            Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause))
          )
        )
      )
      return Option.some(allowanceChecks)
    }

    if (chainType === "aptos") {
      console.log("Aptos not supported atm")
      return Option.none()
    }

    // Unsupported chain type.
    return Option.none()
  })

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

function handleActionButtonClick() {
  if (Option.isNone(transferSteps)) return

  const currentStep = transferSteps.value[currentPage]

  if (currentStep._tag === "Filling") {
    // Lock the transfer values before proceeding
    if (Option.isNone(lockedTransferStore.get())) {
      const newLockedTransfer = LockedTransfer.fromTransfer(
        transfer.sourceChain,
        transfer.destinationChain,
        transfer.channel,
        transferSteps
      )

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
    goToNextPage()
    return
  }

  if (currentStep._tag === "SubmitInstruction") {
    goToNextPage()
    return
  }
}
</script>

<Card
        divided
        class="w-sm my-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden"
>
  <div class="p-4 w-full">
    <StepProgressBar
            class="w-full"
            currentStep={currentPage + 1}
            totalSteps={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.length),
        Option.getOrElse(() =>
          transferSteps.pipe(
            Option.map((ts) => ts.length),
            Option.getOrElse(() => 1),
          ),
        ),
      )}
            stepDescriptions={lockedTransferStore.get().pipe(
        Option.map((lts) => lts.steps.map(getStepDescription)),
        Option.orElse(() =>
          transferSteps.pipe(Option.map((ts) => ts.map(getStepDescription))),
        ),
        Option.getOrElse(() => ["Configure your transfer"]),
      )}
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
      <FillingPage onContinue={handleActionButtonClick} {actionButtonText}/>

      <!-- Dynamic pages for each step -->
      {#if Option.isSome(lockedTransferStore.get())}
        {#each lockedTransferStore.get().value.steps.slice(1) as step, i}
          {#if step._tag === "ApprovalRequired"}
            <ApprovalPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onApprove={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if step._tag === "SubmitInstruction"}
            <SubmitPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    onSubmit={handleActionButtonClick}
                    {actionButtonText}
            />
          {:else if step._tag === "WaitForIndex"}
            <IndexPage
                    stepIndex={i + 1}
                    onBack={goToPreviousPage}
                    {actionButtonText}
            />
          {/if}
        {/each}
      {/if}
    </div>
  </div>

  {#if showDetails}
    <ShowData/>
  {/if}
</Card>

<!-- Debug info can be hidden in production -->
{#if Option.isSome(lockedTransferStore.get()) || Option.isSome(transferSteps)}
  <div class="mt-4">
    <h3 class="text-lg font-semibold">Current Page: {currentPage}</h3>
    <h4 class="text-md">Steps to complete transfer:</h4>
    <ol class="list-decimal pl-5 mt-2">
      {#each lockedTransferStore
        .get()
        .pipe(Option.map((lts) => lts.steps), Option.orElse(() => transferSteps), Option.getOrElse(() => [],),) as step, index}
        <li class="mb-2" class:font-bold={index === currentPage}>
          {#if step._tag === "Filling"}
            <div>Configure transfer details</div>
          {:else if step._tag === "ApprovalRequired"}
            <div>
              Approve token: <span class="font-mono"
            >{truncate(step.token, 8, "middle")}</span
            >
              <div class="text-sm">
                Current allowance: {step.currentAllowance.toString()}
                <br/>
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
