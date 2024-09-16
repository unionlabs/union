<script lang="ts">
// import Zero from "./numbers/zero.svelte"
// import One from "./numbers/one.svelte"
// import Two from "./numbers/two.svelte"
// import Three from "./numbers/three.svelte"
// import Four from "./numbers/four.svelte"
// import Five from "./numbers/five.svelte"
// import Six from "./numbers/six.svelte"
// import Seven from "./numbers/seven.svelte"
// import Eight from "./numbers/eight.svelte"
// import Nine from "./numbers/nine.svelte"
// import H4 from "$lib/components/typography/H4.svelte"
// import H1 from "$lib/components/typography/H1.svelte"
// import Text from "$lib/components/typography/Text.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import H3 from "$lib/components/typography/H3.svelte"
import H1 from "$lib/components/typography/H1.svelte"

type Props = {
  targetTimestamp: number
}

const { targetTimestamp }: Props = $props()

let hours = $state("00")
let minutes = $state("00")
let seconds = $state("00")

// const components = [Zero, One, Two, Three, Four, Five, Six, Seven, Eight, Nine]

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

<div class="flex flex-col h-svh items-center justify-center gap-3">
  <H1></H1>
  <H2>Ceremony set to begin</H2>
  <H3>2024-09-20 | <span class="text-union-accent-500">10:00 AM</span> (CET)</H3>
  <div class="flex gap-2 justify-center">
    {@render pair(hours, 'h')}
    {@render pair(minutes, 'm')}
    {@render pair(seconds, 's')}
  </div>
</div>
s

{#snippet pair(time: string, timeType: string)}
<div class="flex">
  {#each time.split('') as digit, index (index + time)}
    <div class="flex text-white font-supermolot text-2xl font-semibold">
      <div>{digit}</div>
    </div>
  {/each}
  <div class="!text-union-accent-500 self-end uppercase">{timeType}</div>
</div>
{/snippet}