<script lang="ts">
import Zero from "./numbers/zero.svelte"
import One from "./numbers/one.svelte"
import Two from "./numbers/two.svelte"
import Three from "./numbers/three.svelte"
import Four from "./numbers/four.svelte"
import Five from "./numbers/five.svelte"
import Six from "./numbers/six.svelte"
import Seven from "./numbers/seven.svelte"
import Eight from "./numbers/eight.svelte"
import Nine from "./numbers/nine.svelte"
import H4 from "$lib/components/typography/H4.svelte"

type Props = {
  targetTimestamp: number
}

const { targetTimestamp }: Props = $props()

let hours = $state("00")
let minutes = $state("00")
let seconds = $state("00")

const components = [Zero, One, Two, Three, Four, Five, Six, Seven, Eight, Nine]

let interval: ReturnType<typeof setInterval>

function updateCountdown(): void {
  const now: number = Math.floor(Date.now() / 1000)
  const distance: number = targetTimestamp - now

  if (distance < 0) {
    clearInterval(interval)
    hours = minutes = seconds = "00"
    return
  }

  hours = Math.floor(distance / 3600)
    .toString()
    .padStart(2, "0")
  minutes = Math.floor((distance % 3600) / 60)
    .toString()
    .padStart(2, "0")
  seconds = Math.floor(distance % 60)
    .toString()
    .padStart(2, "0")
}

$effect(() => {
  updateCountdown()
  interval = setInterval(updateCountdown, 1000)

  return () => {
    clearInterval(interval)
  }
})
</script>

<div class="p-8 bg-gradient-to-t from-transparent via-black/70 to-transparent backdrop-blur w-full flex items-center justify-center flex-col min-h-48">
  <div class="flex flex-col md:flex-row  justify-center items-center gap-8">
    {@render pair(hours, 'hours')}
    {@render pair(minutes, 'minutes')}
    {@render pair(seconds, 'seconds')}
  </div>
</div>

{#snippet pair(time: string, timeType: string)}
<div>
  <div class="flex">
    {#each time.split('') as digit, index (index + time)}
      <div class="w-20 flex items-center justify-center text-white rounded mb-2">
        {#if /^[0-9]$/.test(digit)}
          {@const Component = components[parseInt(digit)]}
          <Component/>
        {/if}
      </div>
    {/each}
  </div>
  <H4>{timeType}</H4>
</div>
{/snippet}