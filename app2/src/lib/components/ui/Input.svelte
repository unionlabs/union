<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utils"

type Props = HTMLAttributes<HTMLInputElement> & {
  id: string
  value: string
  label: string
  type?: string
  placeholder?: string
  disabled?: boolean
  required?: boolean
  class?: string
}

let {
  id,
  value,
  label,
  type = "text",
  placeholder = "",
  disabled = false,
  required = false,
  class: className = "",
  ...rest
}: Props = $props()

const classes = cn(
  "w-full p-2 bg-zinc-700 border border-zinc-600 rounded text-zinc-200",
  "focus:outline-none focus:ring-2 focus:ring-sky-500",
  "disabled:opacity-50 disabled:cursor-not-allowed",
  "transition-colors duration-200",
  className
)
</script>

<div class="mb-4">
  <label 
    for={id} 
    class="block mb-1 font-medium text-zinc-300"
  >
    {label}
  </label>
  <input
    {id}
    {type}
    bind:value
    {placeholder}
    {disabled}
    {required}
    {...rest}
    class={classes}
    aria-label={label}
  />
</div>
