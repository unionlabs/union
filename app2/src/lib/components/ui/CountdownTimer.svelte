<script lang="ts">
import NumberFlow from "@number-flow/svelte"
import { onMount, onDestroy } from "svelte"

interface Props {
  targetDate: Date
  class?: string
}

let { targetDate, class: className = "" }: Props = $props()

let days = $state(0)
let hours = $state(0)
let minutes = $state(0)
let seconds = $state(0)
let countdownInterval: NodeJS.Timeout | null = null

function updateCountdown() {
  const now = new Date().getTime()
  const distance = targetDate.getTime() - now

  if (distance > 0) {
    days = Math.floor(distance / (1000 * 60 * 60 * 24))
    hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
    minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60))
    seconds = Math.floor((distance % (1000 * 60)) / 1000)
  } else {
    days = hours = minutes = seconds = 0
  }
}

onMount(() => {
  updateCountdown()
  countdownInterval = setInterval(updateCountdown, 1000)
})

onDestroy(() => {
  if (countdownInterval) {
    clearInterval(countdownInterval)
  }
})
</script>

<div class="flex items-center font-mono text-xs text-zinc-300 -ml-1.5 {className}">
  <span class="w-6 text-right">
    <NumberFlow
      value={days}
      transformTiming={{ duration: 800, easing: "ease-out" }}
      format={{ minimumIntegerDigits: 2 }}
    />
  </span><span>d:</span><span class="w-6 text-right">
    <NumberFlow
      value={hours}
      transformTiming={{ duration: 800, easing: "ease-out" }}
      format={{ minimumIntegerDigits: 2 }}
    />
  </span><span>h:</span><span class="w-6 text-right">
    <NumberFlow
      value={minutes}
      transformTiming={{ duration: 800, easing: "ease-out" }}
      format={{ minimumIntegerDigits: 2 }}
    />
  </span><span>m:</span><span class="w-6 text-right">
    <NumberFlow
      value={seconds}
      transformTiming={{ duration: 800, easing: "ease-out" }}
      format={{ minimumIntegerDigits: 2 }}
    />
  </span><span>s</span>
</div> 