<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLDivElement> & {
  class?: string
  randomWidth?: boolean
}

const { class: className = "", randomWidth: useRandomWidth = false, ...rest }: Props = $props()

function randomWidth() {
  const widths = ["w-16", "w-20", "w-24", "w-28", "w-32"]
  return widths[Math.floor(Math.random() * widths.length)]
}

const classes = cn(
  "animate-pulse rounded-md bg-zinc-800",
  useRandomWidth && randomWidth(),
  className
)
</script>

<div
  class={classes}
  {...rest}
></div>
