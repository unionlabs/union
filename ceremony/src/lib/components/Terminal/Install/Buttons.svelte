<script lang="ts">
import { cn, sleep } from "$lib/utils/utils.ts"
import Button from "$lib/components/Terminal/Button.svelte"
import { onDestroy, onMount } from "svelte"

type Props = {
  trigger: (value: any) => void
  index?: number
  data: Array<{
    text: string
    action: string
  }>
}

let { trigger, data, index = 0, ...props }: Props = $props()

let buttons = $state<Array<HTMLButtonElement | null>>([])
let focusedIndex = $state(index)

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault()
    const direction = event.key === "ArrowDown" ? 1 : -1
    focusedIndex = (focusedIndex + direction + buttons.length) % buttons.length
    const button = buttons[focusedIndex]
    if (button && typeof button.focus === "function") {
      button.focus()
    }
  } else if (event.key === "Enter") {
    event.preventDefault()
    const button = buttons[focusedIndex]
    if (button && typeof button.click === "function") {
      button.click()
    }
  }
}

onMount(async () => {
  focusedIndex = index
  await sleep(300)
  window.addEventListener("keydown", handleKeydown)
})

onDestroy(() => {
  window.removeEventListener("keydown", handleKeydown)
})
</script>

{#each data as btn, index}
  {#key btn}
    <Button
            {...props}
            bind:value={buttons[index]}
            onmouseenter={() => focusedIndex = index}
            class={cn(focusedIndex === index ? "bg-union-accent-500 text-black" : "")}
            onclick={() => trigger(btn.action)}
            focus={focusedIndex === index}
    >
      &gt; {btn.text}
    </Button>
  {/key}
{/each}