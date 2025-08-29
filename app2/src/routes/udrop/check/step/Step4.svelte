<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromiseExit$ } from "$lib/runtime"
import { Effect, Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  onSkip: () => void
  onBack?: () => void
}

let { onNext, onSkip, onBack }: Props = $props()

let shouldConnect = $state(false)
let shouldUpdate = $state(false)

const hasTwitter = $derived(
  Option.match(dashboard.connections.twitter, {
    onNone: () => false,
    onSome: (connected) => connected,
  }),
)

const connectResult = runPromiseExit$(() =>
  shouldConnect
    ? Effect.gen(function*() {
      yield* dashboard.linkIdentity("twitter", "/udrop/check?step=4")
      shouldConnect = false
    })
    : Effect.void
)

const updateResult = runPromiseExit$(() =>
  shouldUpdate
    ? Effect.gen(function*() {
      const twitterData = Option.getOrNull(dashboard.user)?.identities?.find(
        identity => identity.provider === "twitter",
      )

      if (twitterData?.identity_data) {
        const twitter_id = twitterData.identity_data.provider_id || twitterData.identity_data.sub
        const twitter_username = twitterData.identity_data.user_name

        const success = yield* Option.match(dashboard.airdrop, {
          onNone: () => Effect.succeed(false),
          onSome: (store) =>
            Effect.tryPromise(() => store.updateTwitter(twitter_id, twitter_username)),
        })
      }

      shouldUpdate = false

      onNext()
    })
    : Effect.void
)

// Derived states for button loading
const isConnecting = $derived(shouldConnect && Option.isNone(connectResult.current))
const isUpdating = $derived(shouldUpdate && Option.isNone(updateResult.current))

function connectTwitter() {
  if (Option.isNone(connectResult.current)) {
    return
  }
  shouldConnect = true
}

function handleContinue() {
  if (hasTwitter) {
    if (Option.isNone(updateResult.current)) {
      return
    }
    shouldUpdate = true
  } else {
    onNext()
  }
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <div class="flex items-center justify-between">
            <h1 class="text-2xl font-semibold">
              Connect Twitter / X
            </h1>
            <div class="px-2 py-1 bg-blue-500/20 border border-blue-500/30 rounded-md flex items-center justify-center">
              <span class="text-xs font-medium text-blue-400 uppercase">Optional</span>
            </div>
          </div>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Connect your Twitter / X to check your eligibility for Mad Yaps Season 0 and Season 1.
          </p>
        </div>
      </div>

      <!-- Connection Status & Action Buttons -->
      <div class="space-y-3">
        <!-- Twitter Connection Status -->
        <div class="bg-zinc-950/50 rounded-lg p-4 border border-zinc-800">
          {#if hasTwitter}
            <div class="flex items-center gap-3">
              <div class="size-8 rounded-lg bg-black flex items-center justify-center flex-shrink-0">
                <svg
                  class="w-4 h-4 text-white"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="m17.687 3.063l-4.996 5.711l-4.32-5.711H2.112l7.477 9.776l-7.086 8.099h3.034l5.469-6.25l4.78 6.25h6.102l-7.794-10.304l6.625-7.571zm-1.064 16.06L5.654 4.782h1.803l10.846 14.34z" />
                </svg>
              </div>
              <div>
                <div class="text-sm font-medium text-white">Twitter / X</div>
                <div class="text-xs text-zinc-400 font-mono">Connected</div>
              </div>
              <div class="ml-auto">
                <div class="w-4 h-4 bg-accent/20 border border-accent/40 rounded-full flex items-center justify-center">
                  <svg
                    class="w-2 h-2 text-accent"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="3"
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                </div>
              </div>
            </div>
          {:else}
            <div class="flex items-center gap-3">
              <div class="w-8 h-8 rounded-lg bg-zinc-700 flex items-center justify-center flex-shrink-0">
                <svg
                  class="w-4 h-4 text-zinc-400"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="m17.687 3.063l-4.996 5.711l-4.32-5.711H2.112l7.477 9.776l-7.086 8.099h3.034l5.469-6.25l4.78 6.25h6.102l-7.794-10.304l6.625-7.571zm-1.064 16.06L5.654 4.782h1.803l10.846 14.34z" />
                </svg>
              </div>
              <div>
                <div class="text-sm font-medium text-zinc-400">No Twitter connected</div>
                <div class="text-xs text-zinc-500">
                  Connect your Twitter / X to check eligibility
                </div>
              </div>
            </div>
          {/if}
        </div>

        <!-- Action Buttons -->
        {#if !hasTwitter}
          <!-- Primary actions -->
          <div class="flex gap-3">
            <Button
              variant="primary"
              class="flex flex-1 items-center justify-center gap-3"
              disabled={isConnecting}
              onclick={connectTwitter}
            >
              {#if isConnecting}
                <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                </div>
                Loading
              {:else}
                Connect
              {/if}
            </Button>
            <Button
              variant="secondary"
              class="flex flex-1 items-center justify-center gap-3"
              onclick={onSkip}
            >
              Skip
            </Button>
          </div>
        {:else}
          <!-- Connected state -->
          <Button
            variant="primary"
            class="flex w-full items-center justify-center gap-3 {isUpdating ? 'opacity-70 cursor-not-allowed' : ''}"
            disabled={isUpdating}
            onclick={handleContinue}
          >
            {#if isUpdating}
              <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
              </div>
              Loading
            {:else}
              Continue
            {/if}
          </Button>
        {/if}

        <!-- Back button always at bottom -->
        {#if onBack}
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
    <div class="relative w-full h-full flex flex-col p-3 sm:p-4">
      <!-- Mobile Title - shown above the content on mobile -->
      <div class="block lg:hidden mb-4 px-1">
        <div class="flex items-center justify-between">
          <h1 class="text-2xl font-semibold">
            Connect Twitter / X
          </h1>
          <div class="px-2 py-1 bg-blue-500/20 border border-blue-500/30 rounded-md flex items-center justify-center">
            <span class="text-xs font-medium text-blue-400 uppercase">Optional</span>
          </div>
        </div>
        <p class="text-sm text-zinc-400 leading-relaxed mt-3">
          Connect your Twitter / X to check your eligibility for Mad Yaps Season 0 and Season 1.
        </p>
      </div>
      <div class="w-full h-full bg-zinc-950 rounded-lg border border-zinc-800 overflow-hidden flex flex-col relative">
        <!-- Background Image -->
        <div class="absolute inset-0">
          <img
            src="/yaps/yaps-bg.png"
            alt="Yaps Background"
            class="w-full h-full object-cover"
          />
        </div>

        <!-- Centered Foreground Image -->
        <div class="absolute inset-0 flex items-center justify-center">
          <img
            src="/yaps/mad-yaos-square.png"
            alt="Mad Yaos"
            class="w-3/4 h-3/4 object-contain"
          />
        </div>
      </div>
    </div>
  {/snippet}
</StepLayout>
