<script lang="ts">
import { cn } from "$lib/utils"
import type { Snippet } from "svelte"
import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements"

type BaseProps = {
  variant?: "primary" | "secondary" | "danger" | "outline" | "text" | "icon" | "inline"
  selected?: boolean | undefined
  class?: string
  children: Snippet
}

type ButtonProps = BaseProps & HTMLButtonAttributes & {
  href?: never
}

type AnchorProps = BaseProps & HTMLAnchorAttributes & {
  href: string
  type?: never
  disabled?: never
}

type Props = ButtonProps | AnchorProps

const props = $props()

const {
  variant = "primary",
  disabled = false,
  selected = false,
  class: className = "",
  children,
  type,
  href,
  ...rest
} = props

const isLink = href !== undefined

const classes = cn(
  // Base styles
  "inline-flex cursor-pointer items-center gap-2 justify-center rounded-md text-sm font-medium transition-colors",
  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2",
  // Disabled styles (only for buttons)
  !isLink && "disabled:pointer-events-none disabled:opacity-50",
  // Disabled styles for links (using aria-disabled)
  isLink && disabled && "pointer-events-none opacity-50",
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

{#if isLink}
  <a
    {href}
    class={classes}
    aria-disabled={disabled}
    {...rest}
  >
    {@render children()}
  </a>
{:else}
  <button
    type={type || "button"}
    class={classes}
    {disabled}
    {...rest}
  >
    {@render children()}
  </button>
{/if}
