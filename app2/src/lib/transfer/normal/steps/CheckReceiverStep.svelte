<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import SharpWarningIcon from "$lib/components/icons/SharpWarningIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import type { CheckReceiver } from "../steps.ts"
import { Option } from "effect"
import { onDestroy, onMount } from "svelte"

type Props = {
  stepIndex: number
  onBack: () => void
  onSubmit: () => void
  step: CheckReceiver
}

const { step, onBack, onSubmit }: Props = $props()

const receiver = $derived(Option.isSome(step.receiver) ? step.receiver.value : undefined)
const chain = $derived(
  Option.isSome(step.destinationChain) ? step.destinationChain.value : undefined
)

let targetTime = $state(0)
let secondsLeft = $state(0)
let intervalId = $state(0)

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
  targetTime = Date.now() + 5_500
  updateTimer()
  intervalId = window.setInterval(updateTimer, 1000)
}

onMount(() => {
  handleIntersect()
})

onDestroy(() => {
  window.clearInterval(intervalId)
})

const isButtonEnabled = $derived(secondsLeft === 0)

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
      <h3 class="text-lg font-semibold">Confirm Receiver</h3>
      <SharpWarningIcon class="text-yellow-500 self-center" height="3rem" width="3rem" />
      <section>
        <p class="text-sm text-zinc-400">
          <span class="font-bold">
            You are sending to an address that is not your currently connected wallet.
          </span>
          Do you wish to proceed?
        </p>
      </section>
      <section>
        <AddressComponent address={receiver} chain={chain} />
      </section>
    </div>

    <div class="flex justify-between mt-4">
      <Button variant="secondary" onclick={onBack}>Back</Button>
      <Button variant="primary" onclick={onSubmit} disabled={!isButtonEnabled}>
        {buttonText}
      </Button>
    </div>
  {/if}
</div>
