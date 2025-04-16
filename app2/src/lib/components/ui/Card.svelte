<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"
import { fade } from "svelte/transition"

type Props = HTMLAttributes<HTMLDivElement> & {
  children: Snippet
  class?: string
  divided?: boolean
  transition?: boolean
}

const {
  children,
  class: className = "",
  divided = false,
  transition = true,
  ...rest
}: Props = $props()

const classes = cn(
  // Base styles
  "rounded border shadow-sm",
  "dark:border-zinc-800 bg-zinc-925",
  // Conditional padding and dividers
  divided ? "p-0 divide-y divide-zinc-900" : "p-4",
  // Additional classes passed as props
  className
)
</script>

<div
  class={classes}
  {...rest}
  in:fade={{delay: transition ? 100 : 0, duration: transition ? 200 : 0 }}
>
  {@render children()}
</div>
