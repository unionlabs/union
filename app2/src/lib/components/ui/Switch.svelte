<script lang="ts">
import { createEventDispatcher } from "svelte"
import type { HTMLButtonAttributes } from "svelte/elements"

/**
 * A toggle switch component based on Bits UI design
 */
type Props = HTMLButtonAttributes & {
  checked: boolean
  disabled?: boolean
  name?: string
  value?: string
  label?: string
  class?: string
}

const dispatch = createEventDispatcher<{
  change: boolean
}>()

const {
  checked = false,
  disabled = false,
  name = undefined,
  value = undefined,
  label = undefined,
  class: className = "",
  ...rest
}: Props = $props()

// Create a reactive variable for the checked state
let isChecked = $state(checked)

// Update internal state when checked prop changes
$effect(() => {
  isChecked = checked
})

function handleClick(event: Event) {
  // Prevent default behavior
  event.preventDefault()

  if (disabled) {
    return
  }

  const newValue = !isChecked
  isChecked = newValue

  // Dispatch the change event with the new value
  dispatch("change", newValue)
}

// Allow for custom styling while providing defaults
const switchClass =
  `relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-950 disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-accent data-[state=unchecked]:bg-zinc-700 ${className}`

const thumbClass =
  "pointer-events-none block h-4 w-4 rounded-full bg-white shadow-lg ring-0 transition-transform data-[state=checked]:translate-x-4 data-[state=unchecked]:translate-x-0"
</script>

<label
  class="flex items-center space-x-2"
  class:opacity-60={disabled}
>
  <button
    type="button"
    role="switch"
    aria-checked={isChecked}
    data-state={isChecked ? "checked" : "unchecked"}
    class={switchClass}
    {disabled}
    onclick={handleClick}
    {...rest}
  >
    <span
      class={thumbClass}
      data-state={isChecked ? "checked" : "unchecked"}
    ></span>
  </button>
  {#if label}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <span
      class="text-sm cursor-pointer"
      onclick={handleClick}
    >{label}</span>
  {/if}
</label>
