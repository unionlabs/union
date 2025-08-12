<script lang="ts">
import { cn } from "$lib/utils"
import type { Snippet } from "svelte"
import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements"

type ButtonProps = HTMLButtonAttributes & HTMLAnchorAttributes & {
  variant?: "primary" | "secondary" | "danger" | "outline" | "text" | "icon" | "inline"
  selected?: boolean | undefined
  class?: string
  children: Snippet
  href?: string
}

let {
  class: className,
  variant = "primary",
  selected = false,
  href = undefined,
  type = "button",
  disabled = false,
  children,
  ...restProps
}: ButtonProps = $props()

const classes = cn(
  // Base styles
  "inline-flex cursor-pointer items-center gap-2 justify-center rounded-md text-sm font-medium transition-colors",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
  "disabled:pointer-events-none disabled:opacity-50",
  // Variants
  variant === "primary" && [
    "bg-sky-600 border-sky-600 border text-white hover:bg-sky-700",
    "dark:bg-white dark:border-zinc-100 dark:hover:bg-zinc-100 dark:text-black font-bold",
    "focus-visible:ring-accent",
  ],
  variant === "secondary" && [
    "border border-zinc-200 bg-white hover:bg-zinc-100 hover:text-zinc-900 font-bold",
    "dark:border-zinc-800 dark:bg-zinc-950 dark:hover:bg-zinc-800 dark:hover:text-zinc-50",
    "focus-visible:ring-zinc-400",
  ],
  variant === "danger" && [
    "bg-red-500 border-red-400 border text-white hover:bg-red-700 font-bold",
    "focus-visible:ring-red-500",
  ],
  variant === "outline" && [
    "border border-zinc-200 bg-transparent text-zinc-900 hover:bg-zinc-100 font-bold",
    "dark:border-zinc-700 dark:text-zinc-100 dark:hover:bg-zinc-800",
    "focus-visible:ring-zinc-400",
  ],
  variant === "text" && [
    "bg-transparent text-zinc-900 hover:bg-zinc-100 font-bold",
    "dark:text-zinc-100 dark:hover:bg-zinc-800",
    "focus-visible:ring-zinc-400",
  ],
  variant === "icon" && [
    "w-9 h-9 p-0 hover:bg-zinc-800 text-gray-400 hover:text-zinc-50",
    "focus-visible:ring-zinc-400",
  ],
  variant === "inline" && [
    "underline text-accent hover:text-orange-300 transition-colors inline-flex items-center gap-1",
  ],
  // Height and padding
  ["icon", "inline"].includes(variant) ? null : "h-9 px-4 py-2",
  // Additional classes passed as props
  className,
)
</script>

{#if href}
  <a
    class={classes}
    {href}
    aria-disabled={disabled}
    {...restProps}
  >
    {@render children()}
  </a>
{:else}
  <button
    class={classes}
    {type}
    {disabled}
    {...restProps}
  >
    {@render children()}
  </button>
{/if}