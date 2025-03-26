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
  import {Effect, Match, Option} from "effect"
  import { wallets } from "$lib/stores/wallets.svelte"
  import { WETH_DENOMS } from "$lib/constants/weth-denoms.ts"
  import {
    createCosmosToCosmosFungibleAssetOrder,
    createCosmosToEvmFungibleAssetOrder,
    createEvmToEvmFungibleAssetOrder,
    type Instruction
  } from "@unionlabs/sdk/ucs03"
  import { createEvmToCosmosFungibleAssetOrder, Batch } from "@unionlabs/sdk/ucs03"
  import {
    createViemPublicClient,
    ViemPublicClient,
    ViemPublicClientSource,
    readErc20Allowance, EvmChannelDestination, ViemPublicClientDestination
  } from "@unionlabs/sdk/evm"

  import {
    CosmWasmClientDestination,
    createCosmWasmClient,
    CosmosChannelDestination, CosmWasmClientSource
  } from "@unionlabs/sdk/cosmos"
  import {fromHex, toHex, http} from "viem"
  import { truncate } from "$lib/utils/format.ts"
  import {
    type TransferStep,
    Filling,
    ApprovalRequired,
    SubmitInstruction,
    getStepDescription
  } from "./transfer-step.ts"
  import {hexAddressToBech32} from "@unionlabs/client";
  import type {CosmWasmClient} from "@cosmjs/cosmwasm-stargate";

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


    if (transferValue.sourceChain.rpc_type == "evm"){
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

    if (transferValue.sourceChain.rpc_type == "cosmos"){
      return Option.some([
        {
          sender: sender.value,
          receiver: transferValue.receiver,
          baseToken: transferValue.baseToken,
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
      console.log("batch: Starting intentsToBatch")

      if (Option.isNone(ti)) {
        return Option.none()
      }
      if (Option.isNone(transfer.sourceChain)) {
        return Option.none()
      }
      if (Option.isNone(transfer.channel)) {
        return Option.none()
      }
      if (Option.isNone(transfer.destinationChain)) {
        return Option.none()
      }
      if (Option.isNone(transfer.ucs03address)) {
        return Option.none()
      }
      console.log("batch: Starting intentsToBatch")

      // Get chain types
      const source = transfer.sourceChain.value.rpc_type
      const destination = transfer.destinationChain.value.rpc_type
      console.log(`batch: Source chain type: ${source}, Destination chain type: ${destination}`)

      // Set up all potential clients
      let publicClientSource: ViemPublicClient | undefined = undefined;
      let publicClientDestination: ViemPublicClient | undefined = undefined;
      let cosmwasmClientSource: CosmWasmClient | undefined = undefined;
      let cosmwasmClientDestination: CosmWasmClient | undefined = undefined;

      // Set up EVM source client if needed
      if (source === "evm") {
        console.log("batch: Setting up EVM source client")
        const viemChainSource = transfer.sourceChain.value.toViemChain()
        if (Option.isNone(viemChainSource)) {
          console.log("batch: Failed to convert source chain to Viem chain, returning None")
          return Option.none()
        }

        console.log(`batch: Creating Viem public client for source chain: ${viemChainSource.value.name}`)
        publicClientSource = yield* createViemPublicClient({
          chain: viemChainSource.value,
          transport: http()
        })
      }

      // Set up EVM destination client if needed
      if (destination === "evm") {
        console.log("batch: Setting up EVM destination client")
        const viemChainDestination = transfer.destinationChain.value.toViemChain()
        if (Option.isNone(viemChainDestination)) {
          console.log("batch: Failed to convert destination chain to Viem chain, returning None")
          return Option.none()
        }

        console.log(`batch: Creating Viem public client for destination chain: ${viemChainDestination.value.name}`)
        publicClientDestination = yield* createViemPublicClient({
          chain: viemChainDestination.value,
          transport: http()
        })
      }

      // Set up Cosmos source client if needed
      if (source === "cosmos") {
        console.log("batch: Setting up Cosmos source client")
        const cosmwasmRpcSource = transfer.sourceChain.value.getRpcUrl("rpc")
        if (Option.isNone(cosmwasmRpcSource)) {
          console.log("batch: Failed to get Cosmos RPC URL for source chain, returning None")
          return Option.none()
        }

        console.log(`batch: Creating CosmWasm client for source chain with RPC: ${cosmwasmRpcSource.value}`)
        cosmwasmClientSource = yield* createCosmWasmClient(cosmwasmRpcSource.value)
      }

      // Set up Cosmos destination client if needed
      if (destination === "cosmos") {
        console.log("batch: Setting up Cosmos destination client")
        const cosmwasmRpcDestination = transfer.destinationChain.value.getRpcUrl("rpc")
        if (Option.isNone(cosmwasmRpcDestination)) {
          console.log("batch: Failed to get Cosmos RPC URL for destination chain, returning None")
          return Option.none()
        }

        console.log(`batch: Creating CosmWasm client for destination chain with RPC: ${cosmwasmRpcDestination.value}`)
        cosmwasmClientDestination = yield* createCosmWasmClient(cosmwasmRpcDestination.value)
      }

      // Create the batch using pattern matching
      console.log(`batch: Creating batch effect for ${source} -> ${destination}`)
      const batchEffect = Effect.gen(function* () {
        console.log(`batch: Inside batch effect generator for ${source} -> ${destination}`)
        console.log(`batch: Transfer intent value:`, ti.value)

        const orders = yield* Match.value([source, destination]).pipe(
          Match.when(["evm", "cosmos"], () => {
            console.log("batch: Matched EVM -> Cosmos pattern", ti.value)
            return Effect.all([
              Effect.tap(
                createEvmToCosmosFungibleAssetOrder(ti.value[0]),
                order => Effect.sync(() => console.log("batch: First order created", order))
              ),
              Effect.tap(
                createEvmToCosmosFungibleAssetOrder(ti.value[1]),
                order => Effect.sync(() => console.log("batch: Second order created", order))
              )
            ]).pipe(
              Effect.tap(orders => Effect.sync(() => console.log("batch: All orders created", orders))),
              Effect.catchAll(error => {
                console.error("batch: Error creating orders", error.cause);
                return Effect.fail(error);
              })
            );
          }),
          Match.when(["evm", "evm"], () => {
            console.log("batch: Matched EVM -> EVM pattern")
            return Effect.all([
              createEvmToEvmFungibleAssetOrder(ti.value[0]),
              createEvmToEvmFungibleAssetOrder(ti.value[1])
            ])
          }),
          Match.when(["cosmos", "evm"], () => {
            console.log("batch: Matched Cosmos -> EVM pattern")
            return createCosmosToEvmFungibleAssetOrder(ti.value[0])
          }),
          Match.when(["cosmos", "cosmos"], () => {
            console.log("batch: Matched Cosmos -> Cosmos pattern")
            return createCosmosToCosmosFungibleAssetOrder(ti.value[0])
          }),
          Match.orElse(() => {
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
        // Provide all services and let Effect handle dependency resolution
        Effect.provideService(ViemPublicClientSource, { client: publicClientSource }),
        Effect.provideService(ViemPublicClientDestination, { client: publicClientDestination }),
        Effect.provideService(CosmWasmClientSource, { client: cosmwasmClientSource }),
        Effect.provideService(CosmWasmClientDestination, { client: cosmwasmClientDestination }),
        Effect.provideService(CosmosChannelDestination, {
          ucs03address: fromHex(transfer.channel.value.destination_port_id, "string"),
          channelId: transfer.channel.value.destination_channel_id
        }),

        Effect.provideService(EvmChannelDestination, {
          ucs03address: transfer.channel.value.source_port_id,
          channelId: transfer.channel.value.source_channel_id,
        })
      )

        const batchResult = yield* batchEffect
        return Option.some(batchResult)
    })


  const checkAllowances = (ti: typeof transferIntents) =>
    Effect.gen(function* () {
      console.info("Checking allowances");
      if (Option.isNone(ti)) return Option.none();
      if (Option.isNone(transfer.sourceChain)) return Option.none();
      if (Option.isNone(transfer.ucs03address)) return Option.none();

      const sourceChain = transfer.sourceChain.value;
      const chainType = sourceChain.rpc_type;
      const spenderAddress = transfer.ucs03address.value;

      // Get the sender's address for the source chain.
      const sender = wallets.getAddressForChain(sourceChain);


      if (Option.isNone(sender)) return Option.none();


      // Get unique token addresses from the transfer intents.
      const tokenAddresses = [...new Set(ti.value.map(intent => intent.baseToken))];

      if (chainType === "evm") {
        // For EVM chains use the existing logic.
        const viemChain = sourceChain.toViemChain();
        if (Option.isNone(viemChain)) return Option.none();
        const publicClientSource = yield* createViemPublicClient({
          chain: viemChain.value,
          transport: http()
        });
        
        const allowanceChecks = yield* Effect.all(
          tokenAddresses.map(tokenAddress =>
            Effect.gen(function* () {
              const allowance = yield* readErc20Allowance(
                tokenAddress,
                sender.value, // EVM sender address
                spenderAddress
              );
              return { token: tokenAddress, allowance };
            }).pipe(Effect.provideService(ViemPublicClient, { client: publicClientSource }))
          )
        );
        return Option.some(allowanceChecks);
      } else if (chainType === "cosmos") {
        // For Cosmos chains use a CosmWasm client to query CW20 allowances.
        const rpcUrl = sourceChain.getRpcUrl("rpc");
        if (Option.isNone(rpcUrl)) return Option.none();
        const cosmwasmClient = yield* createCosmWasmClient(rpcUrl.value);
        // TODO: also for native tokens there are not any allowances, so for native token scenerio
        // We should not call this function

        // Query each token (assumed to be a CW20 contract) for the allowance.
        const allowanceChecks = yield* Effect.all(
          tokenAddresses.map(tokenAddress =>
            Effect.gen(function* () {
              // TODO: 
              // const allowance = yield* readCw20Allowance(contractAddress, walletAddress, spender).pipe(withClient)
              // use it like this when deployed new ts-sdk
              const owner = yield *sourceChain.toCosmosDisplay(sender.value);
              const result = yield* Effect.tryPromise({
                try: () => cosmwasmClient.queryContractSmart(fromHex(tokenAddress, "string"), {
                  allowance: {
                    owner: owner, 
                    spender: spenderAddress
                  }
                }),
                catch: (e) => console.info("Error: ", e)
              })
              console.info("allowance result: ", result)

              return { token: tokenAddress, allowance: BigInt(result.allowance) };
            }).pipe(Effect.provideService(CosmWasmClientSource, { client: cosmwasmClient }),
            Effect.tapErrorCause(cause => Effect.logError("Predict failed with cause", cause)))
          )
        );
        return Option.some(allowanceChecks);
      } else {
        // Unsupported chain type.
        return Option.none();
      }
    });


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
      return
    }
  }
</script>

<Card divided class="w-sm my-24 relative self-center flex flex-col justify-between min-h-[450px] overflow-hidden">
  <div class="p-4 w-full">
    <StepProgressBar
            class="w-full"
            currentStep={currentPage + 1}
            totalSteps={
        lockedTransferStore.get().pipe(
          Option.map(lts => lts.steps.length),
          Option.getOrElse(() => transferSteps.pipe(Option.map(ts => ts.length), Option.getOrElse(() => 1))))}
            stepDescriptions={lockedTransferStore.get().pipe(
        Option.map(lts => lts.steps.map(getStepDescription)),
        Option.orElse(() => transferSteps.pipe(
          Option.map(ts => ts.map(getStepDescription))
        )),
        Option.getOrElse(() => ["Configure your transfer"])
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
      <FillingPage
              onContinue={handleActionButtonClick}
              actionButtonText={actionButtonText}
      />

      <!-- Dynamic pages for each step -->
      {#if Option.isSome(lockedTransferStore.get())}
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
    <ShowData/>
  {/if}
</Card>

<!-- Debug info can be hidden in production -->
{#if Option.isSome(lockedTransferStore.get()) || Option.isSome(transferSteps)}
  <div class="mt-4">
    <h3 class="text-lg font-semibold">Current Page: {currentPage}</h3>
    <h4 class="text-md">Steps to complete transfer:</h4>
    <ol class="list-decimal pl-5 mt-2">
      {#each lockedTransferStore.get().pipe(
        Option.map(lts => lts.steps),
        Option.orElse(() => transferSteps),
        Option.getOrElse(() => [])
      ) as step, index}
        <li class="mb-2" class:font-bold={index === currentPage}>
          {#if step._tag === "Filling"}
            <div>Configure transfer details</div>
          {:else if step._tag === "ApprovalRequired"}
            <div>
              Approve token: <span class="font-mono">{truncate(step.token, 8, "middle")}</span>
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

{#if transfer.state._tag !== "Empty"}
  <pre>{JSON.stringify(transfer.state, null, 2)}</pre>
{/if}

