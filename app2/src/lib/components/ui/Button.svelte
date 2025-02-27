<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLButtonAttributes } from "svelte/elements"
import type { Snippet } from "svelte"

type Props = HTMLButtonAttributes & {
  variant?: "primary" | "secondary"
  class?: string
  children: Snippet
}

const {
  variant = "primary",
  disabled = false,
  type = "button",
  class: className = "",
  children,
  ...rest
}: Props = $props()

const classes = cn(
  // Base styles
  "inline-flex cursor-pointer items-center gap-2 justify-center rounded-md text-sm font-medium transition-colors",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
  "disabled:pointer-events-none disabled:opacity-50",

  // Variants
  variant === "primary" && [
    "bg-sky-600 border-sky-500 border text-white hover:bg-sky-700",
    "dark:bg-sky-600 dark:hover:bg-sky-700",
    "focus-visible:ring-sky-500"
  ],
  variant === "secondary" && [
    "border border-zinc-200 bg-white hover:bg-zinc-100 hover:text-zinc-900",
    "dark:border-zinc-800 dark:bg-zinc-950 dark:hover:bg-zinc-800 dark:hover:text-zinc-50",
    "focus-visible:ring-zinc-400"
  ],

  // Height and padding
  "h-10 px-4 py-2",

  // Additional classes passed as props
  className
)
</script>

<button
  {type}
  class={classes}
  {disabled}
  {...rest}
>
  {@render children()}
</button>
