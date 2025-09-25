<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { getWagmiConnectorClient } from "$lib/services/evm/clients"
import { Data, Effect, Match, Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onBack?: () => void
}

let { onNext, onBack }: Props = $props()

let claim = $derived(Option.flatMap(dashboard.airdrop, (store) => store.claim))

const U_ADDRESS = "0xba5eD44733953d79717F6269357C77718C8Ba5ed"
const U_SYMBOL = "U"
const U_DECIMALS = 18
const U_IMAGE = "https://app.escher.finance/images/token/e-union.svg"

type AddTokenState = Data.TaggedEnum<{
  Ready: {}
  Adding: {}
  Success: {}
  Error: { message: string }
}>

const AddTokenState = Data.taggedEnum<AddTokenState>()

let addTokenState = $state<AddTokenState>(AddTokenState.Ready())

const isReady = $derived(AddTokenState.$is("Ready")(addTokenState))
const isAdding = $derived(AddTokenState.$is("Adding")(addTokenState))
const isSuccess = $derived(AddTokenState.$is("Success")(addTokenState))
const isError = $derived(AddTokenState.$is("Error")(addTokenState))

// Add token to wallet using wagmi
let shouldAddToken = $state(false)

runPromiseExit$(() =>
  shouldAddToken
    ? Effect.gen(function*() {
      addTokenState = AddTokenState.Adding()

      const connectorClient = yield* getWagmiConnectorClient
      const success = yield* Effect.tryPromise({
        try: async () => {
          // Use the connector's request method directly
          return await connectorClient.request({
            method: "wallet_watchAsset",
            params: {
              type: "ERC20",
              options: {
                address: U_ADDRESS,
                symbol: U_SYMBOL,
                decimals: U_DECIMALS,
                image: U_IMAGE,
              },
            },
          })
        },
        catch: (error) => new Error(`Failed to add token: ${String(error)}`),
      })

      if (success) {
        addTokenState = AddTokenState.Success()
      }

      shouldAddToken = false
      return success
    }).pipe(
      Effect.catchAll(error =>
        Effect.gen(function*() {
          const errorObj = error as any
          const errorMessage = String(errorObj?.message || "")

          if (
            errorMessage.includes("UserRejectedRequestError")
            || errorMessage.includes("User rejected")
          ) {
            addTokenState = AddTokenState.Ready()
          } else {
            const message = "Failed to add token to wallet"
            addTokenState = AddTokenState.Error({ message })
          }

          shouldAddToken = false
          return yield* Effect.succeed(false)
        })
      ),
    )
    : Effect.void
)

function handleAddToken() {
  if (isAdding || shouldAddToken) {
    return
  }
  shouldAddToken = true
}

function handleSkip() {
  onNext()
}

function handleContinue() {
  onNext()
}

function handleRetry() {
  addTokenState = AddTokenState.Ready()
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-2xl font-semibold">
            Add U to Wallet
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            {
              Match.value(addTokenState).pipe(
                Match.when(
                  AddTokenState.$is("Adding"),
                  () => "Adding U to your wallet...",
                ),
                Match.when(
                  AddTokenState.$is("Success"),
                  () => "U has been added to your wallet successfully!",
                ),
                Match.when(
                  AddTokenState.$is("Error"),
                  () =>
                    "There was an error adding U. You can add it manually or skip this step.",
                ),
                Match.when(
                  AddTokenState.$is("Ready"),
                  () =>
                    "Add U to your wallet to easily view your balance and make transfers.",
                ),
                Match.exhaustive,
              )
            }
          </p>
        </div>
      </div>

      <div class="space-y-4">
        <!-- Status box -->
        <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
          <div class="flex items-center gap-3">
            <div class="size-8 rounded-lg {isError ? 'bg-red-500/20 border-red-500/40' : 'bg-accent/20 border-accent/40'} flex items-center justify-center flex-shrink-0">
              {#if isAdding}
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
                    d="M12 4v16m8-8H4"
                  />
                </svg>
              {/if}
            </div>
            <div class="flex-1">
              <div class="text-sm font-medium text-white">
                {
                  Match.value(addTokenState).pipe(
                    Match.when(AddTokenState.$is("Ready"), () => "Ready to add U"),
                    Match.when(AddTokenState.$is("Adding"), () => "Adding U"),
                    Match.when(AddTokenState.$is("Success"), () =>
                      "U Added Successfully"),
                    Match.when(AddTokenState.$is("Error"), () =>
                      "Failed to Add U"),
                    Match.exhaustive,
                  )
                }
              </div>
              <div class="text-xs {isError ? 'text-red-400' : 'text-accent'} mt-1">
                {
                  Match.value(addTokenState).pipe(
                    Match.when(AddTokenState.$is("Ready"), () =>
                      "Add U to your wallet for easy access"),
                    Match.when(AddTokenState.$is("Adding"), () =>
                      "Confirm in your wallet..."),
                    Match.when(AddTokenState.$is("Success"), () =>
                      "You can now see U in your wallet"),
                    Match.when(AddTokenState.$is("Error"), ({ message }) =>
                      message),
                    Match.exhaustive,
                  )
                }
              </div>
            </div>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex gap-3">
          {#if isSuccess}
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={handleContinue}
            >
              Continue to Claim
            </Button>
          {:else if isError}
            <Button
              variant="secondary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={handleRetry}
            >
              Try Again
            </Button>
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={handleSkip}
            >
              Skip & Continue
            </Button>
          {:else}
            <Button
              variant="secondary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={handleSkip}
            >
              Skip
            </Button>
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              disabled={isAdding}
              onclick={handleAddToken}
            >
              {#if isAdding}
                <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                </div>
              {/if}
              Add U
            </Button>
          {/if}
        </div>

        <!-- Back button -->
        {#if !isAdding && onBack}
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
        <h1 class="text-2xl font-semibold">Add U to Wallet</h1>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Add the token to your wallet for easy access.
        </p>
      </div>

      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Union Token Video -->
        <div
          class="w-full h-full flex items-center justify-center"
          style="background-color: #0D2024;"
        >
          <video
            class="w-full h-full object-cover filter grayscale"
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
                <span class="text-3xl font-bold text-accent">U</span>
              </div>
            </div>
          </video>
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
