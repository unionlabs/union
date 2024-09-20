<script lang="ts">
import Spinner from "$lib/components/Spinner.svelte"
import type { HTMLButtonAttributes, HTMLAnchorAttributes } from "svelte/elements"

interface Props extends HTMLButtonAttributes, HTMLAnchorAttributes {
  children?: any
  class?: string
  loading?: boolean
  variant?: "primary" | "secondary"
  href?: string
}

let {
  children,
  class: className = "",
  loading = false,
  variant = "primary",
  href,
  ...props
}: Props = $props()

const styles = {
  primary: "bg-union-accent-500 text-black hover:text-black",
  secondary: "bg-transparent text-white border-2 border-white hover:bg-neutral-800"
}
const getClass = (type: "primary" | "secondary") => styles[type] || styles.primary
let combinedClasses = $derived(
  `flex items-center w-fit gap-2 px-4 py-2 font-bold uppercase justify-center ${getClass(variant)} ${className}`
)
</script>

{#if href}
  <a {href} class={combinedClasses} {...props}>
    {@render children()}
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </a>
{:else}
  <button class={combinedClasses} {...props}>
    {@render children()}
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </button>
{/if}