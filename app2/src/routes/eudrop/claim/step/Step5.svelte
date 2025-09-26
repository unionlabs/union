<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { EUDROP_ABI, EUDROP_CONTRACT_ADDRESS } from "$lib/constants/eudrop"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { wallets } from "$lib/stores/wallets.svelte"
import { Evm } from "@unionlabs/sdk-evm"
import { Data, Effect, Layer, Match, Option } from "effect"
import { custom, formatUnits } from "viem"
import { mainnet } from "viem/chains"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

let alreadyClaimed = $state<boolean>(false)
let checkingClaimed = $state<boolean>(false)

interface ClaimParams {
  beneficiary: `0x${string}`
  amount: string
  proof: readonly `0x${string}`[]
}

type ClaimState = Data.TaggedEnum<{
  Ready: {}
  Claiming: {}
  Success: { txHash: string }
  Error: { message: string }
}>

const ClaimState = Data.taggedEnum<ClaimState>()

let claimState = $state<ClaimState>(ClaimState.Ready())

let claim = $derived(Option.flatMap(dashboard.airdrop, (store) => store.claim))

const isClaiming = $derived(ClaimState.$is("Claiming")(claimState))
const isSuccess = $derived(ClaimState.$is("Success")(claimState))
const isError = $derived(ClaimState.$is("Error")(claimState))

let claimAmount = $derived(
  Option.match(claim, {
    onNone: () => "0",
    onSome: (claimData) => claimData.amount ? formatUnits(BigInt(claimData.amount), 18) : "0",
  }),
)

let connectedAddress = $derived(
  Option.match(wallets.evmAddress, {
    onNone: () => "No EVM wallet connected",
    onSome: (addr) => `${addr.address.slice(0, 6)}...${addr.address.slice(-4)}`,
  }),
)

let claimParams = $derived<Option.Option<ClaimParams>>(
  Option.flatMap(claim, (claimData) => {
    if (!claimData.beneficiary || !claimData.amount) {
      return Option.none()
    }
    return Option.some({
      beneficiary: claimData.beneficiary as `0x${string}`,
      amount: claimData.amount.toString(),
      proof: claimData.proof as readonly `0x${string}`[],
    })
  }),
)

let shouldClaim = $state(false)

runPromiseExit$(() =>
  shouldClaim
    ? Effect.gen(function*() {
      yield* Effect.log("Starting claim process")
      claimState = ClaimState.Claiming()

      yield* Effect.log("Getting wallet connector client")
      const connectorClient = yield* getWagmiConnectorClient

      // Get claim parameters first
      const params = yield* Option.match(claimParams, {
        onNone: () => Effect.fail(new Error("No claim data available")),
        onSome: (p) => Effect.succeed(p),
      })

      yield* Effect.log("Switching to mainnet")
      yield* switchChain(mainnet)

      yield* Effect.log("Creating public and wallet clients")
      const publicClient = Evm.PublicClient.Live({
        chain: mainnet,
        transport: custom(connectorClient),
      })

      const walletClient = Evm.WalletClient.Live({
        account: connectorClient.account,
        chain: mainnet,
        transport: custom(connectorClient),
      })

      checkingClaimed = true
      yield* Effect.log("Checking if already claimed")

      const isClaimed = yield* Evm.readContract({
        address: EUDROP_CONTRACT_ADDRESS,
        abi: EUDROP_ABI,
        functionName: "claimed",
        args: [params.beneficiary],
      }).pipe(Effect.provide(publicClient))

      checkingClaimed = false

      if (isClaimed) {
        alreadyClaimed = true
        claimState = ClaimState.Ready()
        return yield* Effect.fail(new Error("Tokens already claimed for this address"))
      }

      yield* Effect.log("Executing claim transaction", {
        beneficiary: params.beneficiary,
        amount: params.amount,
        contract: EUDROP_CONTRACT_ADDRESS,
      })

      // Execute claim transaction
      const txHash = yield* Evm.writeContract({
        address: EUDROP_CONTRACT_ADDRESS,
        abi: EUDROP_ABI,
        functionName: "claim",
        account: connectorClient.account,
        chain: mainnet,
        args: [
          params.beneficiary,
          BigInt(params.amount),
          params.proof,
        ],
      }).pipe(
        Effect.provide(Layer.mergeAll(
          publicClient,
          walletClient,
        )),
      )

      yield* Effect.log("Transaction submitted", { txHash })

      // Wait for receipt
      yield* Effect.log("Waiting for transaction receipt")
      const receipt = yield* Evm.waitForTransactionReceipt(txHash).pipe(
        Effect.provide(publicClient),
      )

      yield* Effect.log("Transaction confirmed", {
        txHash,
        blockNumber: receipt.blockNumber.toString(),
        status: receipt.status,
      })

      // Store transaction hash for Step5
      if (typeof window !== "undefined") {
        localStorage.setItem("lastClaimTxHash", txHash)
      }

      claimState = ClaimState.Success({ txHash })

      yield* Effect.log("Waiting before redirect to Step5")
      yield* Effect.sleep("2 seconds")
      onNext()

      shouldClaim = false
      return { txHash, receipt }
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          // Log the full error for debugging
          console.error("Claim error:", error)

          const errorObj = error as any
          const fullError = errorObj?.cause?.cause?.shortMessage
            || errorObj?.cause?.message
            || errorObj?.message
            || JSON.stringify(error)
          const shortMessage = String(fullError).split(".")[0]

          claimState = ClaimState.Error({ message: shortMessage })
          shouldClaim = false
          checkingClaimed = false
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

function handleClaim() {
  if (isClaiming || shouldClaim) {
    return
  }
  shouldClaim = true
}

function handleRetry() {
  claimState = ClaimState.Ready()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            Claim your eU
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            {
              Match.value(claimState).pipe(
                Match.when(ClaimState.$is("Claiming"), () =>
                  "Please confirm the transaction in your EVM wallet and wait for blockchain confirmation."),
                Match.when(ClaimState.$is("Success"), () =>
                  "Transaction successful! Redirecting to confirmation..."),
                Match.when(ClaimState.$is("Error"), () =>
                  "There was an error processing your claim transaction. Please try again."),
                Match.when(ClaimState.$is("Ready"), () =>
                  "Execute the claim transaction on EVM mainnet to receive your allocated eU."),
                Match.exhaustive,
              )
            }
          </p>
        </div>
      </div>

      <div class="space-y-4">
        <!-- Already Claimed Warning -->
        {#if alreadyClaimed}
          <div class="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4">
            <div class="flex items-start gap-3">
              <div class="w-8 h-8 bg-orange-500/20 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                <svg
                  class="w-4 h-4 text-orange-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-orange-400 mb-2">eU Already Claimed</div>
                <div class="text-xs text-zinc-400">
                  This address has already claimed {claimAmount} eU from this airdrop. Each address
                  can only claim once.
                </div>
              </div>
            </div>
          </div>

          <!-- Back to Dashboard button for already claimed -->
          <Button
            variant="primary"
            class="flex w-full items-center justify-center gap-3"
            href="/dashboard"
          >
            Back to Dashboard
          </Button>
        {/if}

        <!-- Status box - only show when not already claimed -->
        {#if !alreadyClaimed}
          <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
            <div class="flex items-center gap-3">
              <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : alreadyClaimed ? 'bg-orange-500/20 border-orange-500/40' : 'bg-accent/20 border-accent/40'} flex items-center justify-center flex-shrink-0">
                {#if alreadyClaimed === null}
                  <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                  </div>
                {:else if alreadyClaimed}
                  <svg
                    class="w-4 h-4 text-orange-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                {:else if isClaiming}
                  <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                  </div>
                {:else if isError}
                  <svg
                    class="w-4 h-4 text-red-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 9v2m0 4h.01"
                    />
                  </svg>
                {:else if isSuccess}
                  <svg
                    class="w-4 h-4 text-accent"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                {:else}
                  <svg
                    class="w-4 h-4 text-accent"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1"
                    />
                  </svg>
                {/if}
              </div>
              <div class="flex-1">
                <div class="text-sm font-medium text-white">
                  {#if alreadyClaimed === null}
                    Checking Claim Status
                  {:else if alreadyClaimed}
                    Already Claimed
                  {:else}
                    {
                      Match.value(claimState).pipe(
                        Match.when(ClaimState.$is("Ready"), () => "Ready to Claim"),
                        Match.when(ClaimState.$is("Claiming"), () => "Claiming Tokens"),
                        Match.when(ClaimState.$is("Success"), () =>
                          "Transaction Successful"),
                        Match.when(ClaimState.$is("Error"), () =>
                          "Claim Failed"),
                        Match.exhaustive,
                      )
                    }
                  {/if}
                </div>
                <div class="text-xs {isError ? 'text-red-400' : alreadyClaimed ? 'text-orange-400' : 'text-accent'} mt-1">
                  {#if alreadyClaimed === null}
                    Verifying onchain status...
                  {:else if alreadyClaimed}
                    This address has already claimed eU
                  {:else}
                    {
                      Match.value(claimState).pipe(
                        Match.when(ClaimState.$is("Ready"), () =>
                          `${claimAmount} eU to ${connectedAddress}`),
                        Match.when(ClaimState.$is("Claiming"), () =>
                          "Confirm transaction in your Ethereum wallet"),
                        Match.when(ClaimState.$is("Success"), () =>
                          "Preparing confirmation..."),
                        Match.when(ClaimState.$is("Error"), ({ message }) =>
                          message),
                        Match.exhaustive,
                      )
                    }
                  {/if}
                </div>
              </div>
            </div>
          </div>
        {/if}

        <!-- Button - hide if already claimed -->
        {#if !alreadyClaimed}
          <Button
            variant={isError ? "secondary" : "primary"}
            class="flex w-full items-center justify-center gap-3"
            disabled={isClaiming || isSuccess || checkingClaimed}
            onclick={isError ? handleRetry : handleClaim}
          >
            {#if isClaiming}
              <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
              </div>
            {:else if isSuccess}
              <svg
                class="w-4 h-4 text-current"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M5 13l4 4L19 7"
                />
              </svg>
            {/if}
            {
              Match.value(claimState).pipe(
                Match.when(ClaimState.$is("Ready"), () => `Claim ${claimAmount} eU`),
                Match.when(ClaimState.$is("Claiming"), () => "Claiming..."),
                Match.when(ClaimState.$is("Success"), () => "Success!"),
                Match.when(ClaimState.$is("Error"), () => "Try Again"),
                Match.exhaustive,
              )
            }
          </Button>
        {/if}

        <!-- Back button (only show when not claiming, successful, or already claimed) -->
        {#if !isClaiming && !isSuccess && !alreadyClaimed && onBack}
          <Button
            variant="secondary"
            class="flex w-full items-center justify-center gap-3"
            onclick={onBack}
          >
            ← Back
          </Button>
        {/if}
      </div>
    </div>
  {/snippet}

  {#snippet right()}
    <div class="relative w-full h-full flex flex-col p-4">
      <!-- Mobile Title -->
      <div class="block lg:hidden mb-4">
        <h1 class="text-2xl font-semibold">
          Claim your eU
        </h1>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Union Token Video - all states use the same video -->
        <div
          class="w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <video
            class="w-full h-full object-cover"
            autoplay
            loop
            muted
            playsinline
          >
            <source
              src="https://videos.cdn.union.build/spin-token.webm"
              type="video/webm"
            >
            <!-- Fallback for browsers that don't support the video -->
            <div class="w-full h-full flex items-center justify-center">
              <div class="w-24 h-24 bg-accent/20 rounded-full flex items-center justify-center border-4 border-accent">
                <span class="text-3xl font-bold text-accent">eU</span>
              </div>
            </div>
          </video>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
