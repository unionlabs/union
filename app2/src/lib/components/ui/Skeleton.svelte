<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLDivElement> & {
  class?: string
  randomWidth?: boolean
}

const { class: className = "", randomWidth: useRandomWidth = false, ...rest }: Props = $props()

const widths = ["4rem", "5rem", "6rem", "7rem", "8rem"]
let currentWidth = $state(widths[Math.floor(Math.random() * widths.length)])

$effect(() => {
  if (!useRandomWidth) return
  const interval = setInterval(() => {
    let newWidth: string
    do {
      newWidth = widths[Math.floor(Math.random() * widths.length)]
    } while (newWidth === currentWidth)
    currentWidth = newWidth
  }, 1000)
  return () => clearInterval(interval)
})

const classes = cn("animate-pulse rounded-md bg-zinc-800 transition-all duration-300", className)

const style = $derived(useRandomWidth ? `width: ${currentWidth}` : undefined)
</script>

<div
  class={classes}
  style={style}
  {...rest}
></div>
