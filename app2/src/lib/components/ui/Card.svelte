<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"

type Props = HTMLAttributes<HTMLDivElement> & {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()

const classes = cn(
  // Base styles
  "rounded p-4 border shadow-sm",
  "dark:border-zinc-700 dark:bg-zinc-900",
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
