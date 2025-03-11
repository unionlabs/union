<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  children: Snippet
  class?: string
  divided?: boolean
}

const { children, class: className = "", divided = false, ...rest }: Props = $props()

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
>
  {@render children()}
</div>
