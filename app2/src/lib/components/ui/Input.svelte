<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLInputAttributes } from "svelte/elements"
import Label from "./Label.svelte"

type Props = HTMLInputAttributes & {
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
  "w-full p-2 bg-zinc-900 rounded-md text-zinc-500",
  "focus:outline-none focus:ring-1 focus:ring-accent",
  "disabled:opacity-50 disabled:cursor-not-allowed",
  "transition-colors duration-200",
  className,
)
</script>

<div>
  <Label for={id}>{label}</Label>
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
