<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { Effect, Option } from "effect"
import StepLayout from "../StepLayout.svelte"

interface Props {
  onNext: () => void
  isActive: boolean
}

let { onNext, isActive }: Props = $props()

let isLoadingClaim = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => true,
    onSome: (store) => store.isLoadingClaim,
  }),
)

let hasClaim = $derived(
  Option.match(dashboard.airdrop, {
    onNone: () => false,
    onSome: (store) => store.hasClaim,
  }),
)

type AuthProvider = {
  id: "twitter" | "github" | "discord"
  name: string
  icon: string
  iconColor: string
}

const providers: Array<AuthProvider> = [
  {
    id: "twitter",
    name: "Twitter",
    icon:
      `<path fill="currentColor" d="m17.687 3.063l-4.996 5.711l-4.32-5.711H2.112l7.477 9.776l-7.086 8.099h3.034l5.469-6.25l4.78 6.25h6.102l-7.794-10.304l6.625-7.571zm-1.064 16.06L5.654 4.782h1.803l10.846 14.34z"/>`,
    iconColor: "text-white",
  },
  {
    id: "github",
    name: "GitHub",
    icon:
      `<path fill="currentColor" fill-rule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clip-rule="evenodd"/>`,
    iconColor: "text-white",
  },
  {
    id: "discord",
    name: "Discord",
    icon:
      `<path fill="currentColor" d="M20.317 4.37a19.791 19.791 0 00-4.885-1.515.074.074 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 00-5.487 0 12.64 12.64 0 00-.617-1.25.077.077 0 00-.079-.037A19.736 19.736 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 00.031.057 19.9 19.9 0 005.993 3.03.078.078 0 00.084-.028c.462-.63.874-1.295 1.226-1.994a.076.076 0 00-.041-.106 13.107 13.107 0 01-1.872-.892.077.077 0 01-.008-.128 10.2 10.2 0 00.372-.292.074.074 0 01.077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 01.078.01c.12.098.246.198.373.292a.077.077 0 01-.006.127 12.299 12.299 0 01-1.873.892.077.077 0 00-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 00.084.028 19.839 19.839 0 006.002-3.03.077.077 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 00-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"/>`,
    iconColor: "text-white",
  },
] as const

let loading = $state(false)

function handleLogin(provider: AuthProvider) {
  if (loading) {
    return
  }
  loading = true

  runPromise(
    Effect.gen(function*() {
      yield* dashboard.login(provider.id, "/udrop/claim")
    }).pipe(
      Effect.tap(() => Effect.sync(() => loading = false)),
      Effect.catchAll(() => Effect.sync(() => loading = false)),
    ),
  )
}

function handleContinue() {
  if (Option.isSome(dashboard.session)) {
    onNext()
  }
}
</script>

<StepLayout>
  {#snippet left()}
    <div class="flex flex-col gap-6 p-4 z-10 justify-between h-full">
      <div class="space-y-4 hidden lg:block">
        <div>
          <h1 class="text-4xl font-semibold">
            Claim Your eU
          </h1>
          <p class="text-sm text-zinc-400 leading-relaxed mt-3">
            Authenticate to access your pre-generated claim proof and complete your token claim.
          </p>
        </div>
      </div>

      {#if Option.isSome(dashboard.session)}
        <div class="space-y-3">
          {#if isLoadingClaim}
            <div class="bg-accent/10 border border-accent/20 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 bg-accent/20 rounded-full flex items-center justify-center">
                  <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin">
                  </div>
                </div>
                <div>
                  <div class="text-sm font-medium text-accent">Checking Claim Status</div>
                  <div class="text-xs text-zinc-400">
                    Please wait while we verify your eligibility...
                  </div>
                </div>
              </div>
            </div>
          {:else if hasClaim && isActive}
            <div class="bg-accent/10 border border-accent/20 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 bg-accent/20 rounded-full flex items-center justify-center">
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
                </div>
                <div>
                  <div class="text-sm font-medium text-accent">Claim Available!</div>
                  <div class="text-xs text-zinc-400">You're eligible to claim eU</div>
                </div>
              </div>
            </div>
            <Button
              variant="primary"
              class="flex w-full items-center justify-center gap-3"
              onclick={handleContinue}
            >
              Continue to Claim
            </Button>
          {:else if hasClaim && !isActive}
            <div class="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 bg-orange-500/20 rounded-full flex items-center justify-center">
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
                      d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                </div>
                <div>
                  <div class="text-sm font-medium text-orange-400">Airdrop Not Active</div>
                  <div class="text-xs text-zinc-400">
                    The airdrop claiming is currently disabled
                  </div>
                </div>
              </div>
            </div>
            <Button
              variant="primary"
              class="flex w-full items-center justify-center gap-3"
              href="/dashboard"
            >
              Back to Dashboard
            </Button>
          {:else}
            <div class="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="w-8 h-8 bg-orange-500/20 rounded-full flex items-center justify-center">
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
                      d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                </div>
                <div>
                  <div class="text-sm font-medium text-orange-400">No Claim Available</div>
                  <div class="text-xs text-zinc-400">
                    You don't have any eU to claim at this time
                  </div>
                </div>
              </div>
            </div>
            <Button
              variant="primary"
              class="flex w-full items-center justify-center gap-3"
              href="/dashboard"
            >
              Back to Dashboard
            </Button>
          {/if}
        </div>
      {:else}
        <div class="space-y-3">
          <div class="flex items-center gap-3 text-sm text-gray-400">
            Authenticate to continue
          </div>
          <div class="space-y-2.5">
            {#each providers as provider (provider.id)}
              <Button
                variant="secondary"
                class="flex w-full items-center justify-center gap-3 relative hover:translate-y-[1px] transition-all {loading ? 'opacity-70 cursor-not-allowed' : ''}"
                disabled={loading}
                onclick={() => handleLogin(provider)}
              >
                <svg
                  class="w-5 h-5 {provider.iconColor} {loading ? 'opacity-70' : ''}"
                  viewBox="0 0 24 24"
                >
                  {@html provider.icon}
                </svg>
                <span class={loading ? "opacity-70" : ""}>
                  {provider.name}
                </span>
                {#if loading}
                  <div class="absolute right-4 w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin">
                  </div>
                {/if}
              </Button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/snippet}

  {#snippet right()}
    <div class="aspect-square relative flex items-center justify-center">
      <!-- Mobile Title - shown above the content on mobile -->
      <div class="block lg:hidden absolute top-4 left-4 right-4 z-10">
        <h1 class="text-4xl font-semibold text-center">
          Claim Your eU
        </h1>
      </div>
      <div class="ball border-primary absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 z-0">
        <span></span>
        <span></span>
        <span></span>
        <span></span>
        <span></span>
        <span></span>
      </div>
      <img
        src="/images/union-logo-glyph.svg"
        alt="Union Logo"
        class="w-24 h-auto border border-zinc-700 bg-zinc-925 rounded-full p-4 pulse-logo relative z-50"
      />
    </div>
  {/snippet}
</StepLayout>

<style>
.ball span{
  width: 0;
  height: 0;
  border-radius: 50%;
  display: block;
  transition: all 2s ease-in-out;
  transform:translate(-50%,-50%);
  position:absolute;
  border: 1px solid transparent;
  animation: ballsAnimationBigger 6s infinite linear;
  pointer-events: none;
}

@keyframes ballsAnimationBigger {
  0% {
    border-color: transparent;
    opacity:0;
  }
  20% {
    border-color: inherit;
  }
  80%{
    border-color: transparent;
    opacity:1;
  }
  100% {
    width: 100vh;
    height: 100vh;
  }
}

.ball span:nth-child(2){
  animation-delay: 1s;
}
.ball span:nth-child(3){
  animation-delay: 2s;
}
.ball span:nth-child(4){
  animation-delay: 3s;
}
.ball span:nth-child(5){
  animation-delay: 4s;
}
.ball span:nth-child(6){
  animation-delay: 5s;
}

.pulse-logo {
  animation: logoPulse 6s infinite ease-in-out;
}

@keyframes logoPulse {
  0% {
    transform: scale(0.95);
  }
  8.33% {
    transform: scale(1.05);
  }
  16.66% {
    transform: scale(0.95);
  }
  25% {
    transform: scale(1.05);
  }
  33.33% {
    transform: scale(0.95);
  }
  41.66% {
    transform: scale(1.05);
  }
  50% {
    transform: scale(0.95);
  }
  58.33% {
    transform: scale(1.05);
  }
  66.66% {
    transform: scale(0.95);
  }
  75% {
    transform: scale(1.05);
  }
  83.33% {
    transform: scale(0.95);
  }
  91.66% {
    transform: scale(1.05);
  }
  100% {
    transform: scale(0.95);
  }
}
</style>
