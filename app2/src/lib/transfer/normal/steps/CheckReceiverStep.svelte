<script lang="ts">
import SharpWarningIcon from "$lib/components/icons/SharpWarningIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Label from "$lib/components/ui/Label.svelte"
import { Option } from "effect"
import { onDestroy, onMount } from "svelte"
import type { CheckReceiver } from "./steps"

type Props = {
  stepIndex: number
  cancel: () => void
  onSubmit: () => void
  step: CheckReceiver
}

const { step, cancel, onSubmit }: Props = $props()

const receiver = $derived(step.receiver)
const chain = $derived(step.destinationChain)

let targetTime = $state(0)
let secondsLeft = $state(0)
let intervalId = $state(0)
let addressConfirmed = $state(false)

const updateTimer = () => {
  const now = Date.now()
  const distance = targetTime - now
  if (distance <= 0) {
    secondsLeft = 0
    clearInterval(intervalId)
    return
  }
  secondsLeft = Math.floor(distance / 1000)
}

const handleIntersect = () => {
  targetTime = Date.now() + 10_000
  updateTimer()
  intervalId = window.setInterval(updateTimer, 1000)
}

onMount(() => {
  handleIntersect()
})

onDestroy(() => {
  window.clearInterval(intervalId)
})

const isButtonEnabled = $derived(secondsLeft === 0 && addressConfirmed)

const buttonText = $derived.by(() => {
  if (secondsLeft > 0) {
    return `Proceed in ${secondsLeft}…`
  }
  return "Proceed"
})
</script>

<div class="relative min-w-full p-4 flex flex-col justify-between h-full">
  {#if receiver && chain}
    <div class="flex-1 flex flex-col gap-4">
      <div class="flex items-center gap-3">
        <div class="bg-amber-500 text-white text-xs font-bold px-2 py-1 rounded-md flex items-center gap-1">
          <SharpWarningIcon class="size-3" />
          External Wallet Address
        </div>
      </div>

      <p class="text-sm text-zinc-400">
        You are sending to an external wallet address that is <span class="font-semibold text-white"
        >NOT your connected wallet</span>. Please verify the wallet address carefully.
      </p>

      <section>
        <Label class="mb-2">Wallet Address</Label>
        <div class="border border-zinc-300 dark:border-zinc-700 rounded-md p-2 px-3">
          <div class="text-white font-mono">
            <AddressComponent
              address={receiver}
              chain={chain}
            />
          </div>
        </div>
      </section>

      <label class="flex items-start cursor-pointer mt-2">
        <input
          type="checkbox"
          class="opacity-0 absolute w-1 h-1 peer"
          bind:checked={addressConfirmed}
          aria-label="Confirm address is correct"
        />
        <span
          class="h-4 w-4 rounded border border-zinc-600 bg-transparent flex items-center justify-center transition-colors peer-checked:border-accent peer-focus:ring-2 peer-focus:ring-accent peer-focus:ring-offset-2 peer-focus:ring-offset-zinc-900 hover:border-zinc-500 mr-2 shrink-0"
        >
          {#if addressConfirmed}
            <svg
              class="w-2.5 h-2.5 text-accent pointer-events-none"
              viewBox="0 0 16 16"
              fill="currentColor"
            >
              <path d="M12.207 4.793a1 1 0 010 1.414l-5 5a1 1 0 01-1.414 0l-2-2a1 1 0 011.414-1.414L6.5 9.086l4.293-4.293a1 1 0 011.414 0z" />
            </svg>
          {/if}
        </span>
        <span class="-mt-1.5 ml-1 text-sm text-zinc-600 dark:text-zinc-400">
          I want to send to this external wallet address. Union <span
            class="font-semibold text-white"
          >cannot recover my funds</span> if this address is incorrect.
        </span>
      </label>
    </div>

    <div class="flex justify-between mt-4">
      <Button
        variant="secondary"
        onclick={cancel}
      >
        Cancel
      </Button>
      <Button
        variant="primary"
        onclick={onSubmit}
        disabled={!isButtonEnabled}
      >
        {buttonText}
      </Button>
    </div>
  {/if}
</div>
