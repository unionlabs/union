<script lang="ts">
import { AccountError } from "$lib/dashboard/errors"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { runPromise } from "$lib/runtime"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Effect, Option, pipe } from "effect"
import { errorStore } from "../stores/errors.svelte"

interface Props {
  isOpen: boolean
  onClose: () => void
}

let { isOpen, onClose }: Props = $props()

let step: "initial" | "confirm" = $state("initial")
let userInput = $state("")
let countdown = $state(5)
let isDeleting = $state(false)
let countdownInterval: number | null = null

const userId = $derived(
  Option.isSome(dashboard.user) ? dashboard.user.value.id : "",
)

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

function handleDelete() {
  const effect = pipe(
    Effect.sync(() => {
      isDeleting = true
    }),
    Effect.flatMap(() => dashboard.deleteAccount()),
    Effect.catchAll((error) => {
      errorStore.showError(
        new AccountError({
          cause: extractErrorDetails(error),
          operation: "delete",
        }),
      )
      return Effect.succeed(undefined)
    }),
    Effect.ensuring(
      Effect.sync(() => {
        isDeleting = false
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

{#if isOpen}
  <div class="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-zinc-900 border border-zinc-800 rounded-lg max-w-md w-full p-6">
      {#if step === "initial"}
        <h2 class="text-xl font-semibold text-white mb-4">Delete Account</h2>

        <div class="bg-rose-950/30 border border-rose-900/50 rounded-lg p-4 mb-6">
          <p class="text-rose-400 font-medium mb-2">⚠️ SERIOUS WARNING</p>
          <p class="text-zinc-300 text-sm">
            This will permanently delete your account. You will lose everything. You will lose all
            of your XP, your rank, and all of your achievements.
          </p>
        </div>

        <p class="text-zinc-400 text-sm mb-4">
          To proceed, please type your account ID exactly as shown below:
        </p>

        <div class="bg-zinc-800/50 rounded p-3 mb-4">
          <code class="text-zinc-200 text-sm font-mono select-none">{userId}</code>
        </div>

        <input
          type="text"
          bind:value={userInput}
          onpaste={handlePaste}
          placeholder="Type your account ID here"
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
            disabled={userInput !== userId}
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
            You are about to permanently delete your account. All your data, settings, and history
            will be lost forever. This cannot be undone. You will lose all of your XP, your rank,
            and any achievements. You will never get this back.
          </p>
        </div>

        <p class="text-zinc-400 text-sm mb-6">
          {#if countdown > 0}
            Please wait {countdown} seconds before you can delete your account.
          {:else}
            You can now delete your account.
          {/if}
        </p>

        <div class="flex gap-3">
          <button
            onclick={handleClose}
            disabled={isDeleting}
            class="flex-1 px-4 py-2 bg-zinc-800 text-zinc-300 rounded hover:bg-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleDelete}
            disabled={countdown > 0 || isDeleting}
            class="flex-1 px-4 py-2 bg-rose-600 text-white rounded hover:bg-rose-700 disabled:bg-zinc-800 disabled:text-zinc-500 disabled:cursor-not-allowed transition-colors"
          >
            {isDeleting ? "Deleting..." : "Delete Account"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}
