<script lang="ts">
import { AccountError } from "$lib/dashboard/errors"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Effect, Option, pipe } from "effect"
import { errorStore } from "../stores/errors.svelte"
import type { AuthProvider } from "../stores/user.svelte"

interface Props {
  isOpen: boolean
  onClose: () => void
  provider: AuthProvider | null
}

let { isOpen, onClose, provider }: Props = $props()

let step: "initial" | "confirm" = $state("initial")
let userInput = $state("")
let countdown = $state(5)
let isUnlinking = $state(false)
let countdownInterval: number | null = null

const providerName = $derived(
  provider ? provider.charAt(0).toUpperCase() + provider.slice(1) : "",
)

const confirmText = $derived(provider ? `unlink ${provider} and delete my xp` : "")

function resetModal() {
  step = "initial"
  userInput = ""
  countdown = 5
  if (countdownInterval) {
    clearInterval(countdownInterval)
    countdownInterval = null
  }
}

function handleClose() {
  resetModal()
  onClose()
}

function proceedToConfirm() {
  step = "confirm"
  startCountdown()
}

function startCountdown() {
  countdownInterval = window.setInterval(() => {
    countdown--
    if (countdown <= 0 && countdownInterval) {
      clearInterval(countdownInterval)
      countdownInterval = null
    }
  }, 1000)
}

function handlePaste(e: Event) {
  e.preventDefault()
  return false
}

function handleUnlink() {
  if (!provider) {
    return
  }

  const effect = pipe(
    Effect.sync(() => {
      isUnlinking = true
    }),
    Effect.flatMap(() => dashboard.unlinkIdentity(provider)),
    Effect.catchAll((error) => {
      errorStore.showError(
        new AccountError({
          cause: extractErrorDetails(error),
          operation: "unlink",
        }),
      )
      return Effect.succeed(undefined)
    }),
    Effect.ensuring(
      Effect.sync(() => {
        isUnlinking = false
        handleClose()
      }),
    ),
  )

  return runPromise(effect)
}

$effect(() => {
  if (!isOpen) {
    resetModal()
  }
})
</script>

{#if isOpen && provider}
  <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-lg max-w-md w-full p-6">
      {#if step === "initial"}
        <h2 class="text-xl font-semibold text-white mb-4">Unlink {providerName} Account</h2>

        <div class="bg-rose-950/30 border border-rose-900/50 rounded-lg p-4 mb-6">
          <p class="text-rose-400 font-medium mb-2">⚠️ WARNING</p>
          <p class="text-zinc-300 text-sm">
            Unlinking your {providerName} account will permanently remove all rewards and XP
            associated with this linked account. This action cannot be undone and you will lose all
            progress tied to this connection.
          </p>
        </div>

        <p class="text-zinc-400 text-sm mb-4">
          To proceed, please type <span class="font-mono bg-zinc-800 px-2 py-1 rounded">{
            confirmText
          }</span> exactly as shown:
        </p>

        <input
          type="text"
          bind:value={userInput}
          onpaste={handlePaste}
          placeholder="Type the confirmation text here"
          class="w-full bg-zinc-800 border border-zinc-700 rounded px-3 py-2 text-zinc-200 placeholder:text-zinc-500 focus:outline-none focus:border-zinc-600 mb-4"
        />

        <div class="flex gap-3">
          <button
            onclick={handleClose}
            class="flex-1 px-4 py-2 bg-zinc-800 text-zinc-300 rounded hover:bg-zinc-700 transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={proceedToConfirm}
            disabled={userInput !== confirmText}
            class="flex-1 px-4 py-2 bg-rose-600 text-white rounded hover:bg-rose-700 disabled:bg-zinc-800 disabled:text-zinc-500 disabled:cursor-not-allowed transition-colors"
          >
            Continue
          </button>
        </div>
      {/if}

      {#if step === "confirm"}
        <h2 class="text-xl font-semibold text-white mb-4">Final Confirmation</h2>

        <div class="bg-rose-950/30 border border-rose-900/50 rounded-lg p-4 mb-6">
          <p class="text-rose-400 font-medium mb-2">⚠️ THIS ACTION CANNOT BE UNDONE</p>
          <p class="text-zinc-300 text-sm">
            You are about to permanently unlink your {providerName} account. All rewards, XP, and
            achievements associated with this linked account will be lost forever. You will need to
            reconnect this account later if you want to use it again, but your previous rewards and
            progress will not be restored.
          </p>
        </div>

        <p class="text-zinc-400 text-sm mb-6">
          {#if countdown > 0}
            Please wait {countdown} seconds before you can unlink your account.
          {:else}
            You can now unlink your {providerName} account.
          {/if}
        </p>

        <div class="flex gap-3">
          <button
            onclick={handleClose}
            disabled={isUnlinking}
            class="flex-1 px-4 py-2 bg-zinc-800 text-zinc-300 rounded hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleUnlink}
            disabled={countdown > 0 || isUnlinking}
            class="flex-1 px-4 py-2 bg-rose-600 text-white rounded hover:bg-rose-700 disabled:bg-zinc-800 disabled:text-zinc-500 disabled:cursor-not-allowed transition-colors"
          >
            {isUnlinking ? "Unlinking..." : `Unlink ${providerName}`}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
