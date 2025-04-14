<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { Array as Arr, Option, Struct } from "effect"
import { lockedTransferStore } from "../locked-transfer.svelte.ts"
import { is } from "../transfer-step.ts"
import { onDestroy, onMount } from "svelte"
import SharpWarningIcon from "$lib/components/icons/SharpWarningIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"

type Props = {
  stepIndex: number
  onBack: () => void
  onSubmit: () => void
}

const { stepIndex, onBack, onSubmit }: Props = $props()

const lts = lockedTransferStore.get()

let element: HTMLElement
let observer: IntersectionObserver | null

const step = $derived(
  lts.pipe(
    Option.map(Struct.get("steps")),
    Option.flatMap(Arr.get(stepIndex)),
    Option.filter(is("CheckReceiver"))
  )
)

const receiver = $derived(Option.map(step, x => x.receiver))
const chain = $derived(Option.map(step, x => x.destinationChain))

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
  observer = new IntersectionObserver(handleIntersect, {
    root: null,
    rootMargin: "0px",
    threshold: 0.1 // adjust threshold as needed
  })
  observer.observe(element)
})

onDestroy(() => {
  if (observer && element) {
    observer.unobserve(element)
  }
  window.clearInterval(intervalId)
})

const isButtonEnabled = $derived(secondsLeft === 0)

const buttonText = $derived.by(() => {
  if (secondsLeft > 0) {
    return `Proceed in ${secondsLeft}…`
  }
  return "Proceed"
})

$effect(() => {
  console.log({ chain, receiver })
})
</script>

<div class="relative min-w-full p-4 flex flex-col justify-between h-full">
  {#if Option.isSome(chain) && Option.isSome(receiver)}
    <div class="flex-1 flex flex-col gap-4">
      <h3 class="text-lg font-semibold">Confirm Receiver</h3>
      <SharpWarningIcon
        class="text-yellow-500 self-center"
        height="3rem"
        width="3rem"
      />
      <section>
        <p class="text-sm text-zinc-400">
          <span class="font-bold">
            You are sending to an address that is not your currently connected
            wallet.
          </span>
          Do you wish to proceed?
        </p>
      </section>
      <section>
        <AddressComponent
          address={receiver.value.value}
          chain={chain.value.value}
        />
      </section>
    </div>

    <div class="flex justify-between mt-4" bind:this={element}>
      <Button variant="secondary" onclick={onBack}>Back</Button>
      <Button variant="primary" onclick={onSubmit} disabled={!isButtonEnabled}>
        {buttonText}
      </Button>
    </div>
  {/if}
</div>
