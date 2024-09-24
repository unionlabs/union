<script lang="ts">
import Spinner from "$lib/components/Spinner.svelte"
import type { HTMLButtonAttributes, HTMLAnchorAttributes } from "svelte/elements"

type CommonProps = {
  children?: any
  class?: string
  loading?: boolean
  variant?: "primary" | "secondary"
}

type ButtonProps = CommonProps & Omit<HTMLButtonAttributes, keyof CommonProps>
type AnchorProps = CommonProps & Omit<HTMLAnchorAttributes, keyof CommonProps> & { href: string }

type Props = ButtonProps | AnchorProps

let props: Props = $props()

const { children, class: className = "", loading = false, variant = "primary" } = props

const styles = {
  primary:
    "bg-union-accent-500 text-black hover:text-black hover:bg-union-accent-400 disabled:bg-neutral-500",
  secondary:
    "bg-transparent text-white border-2 border-white hover:bg-neutral-800 disabled:bg-neutral-500"
}

const getClass = (type: "primary" | "secondary") => styles[type] || styles.primary
let combinedClasses = $derived(
  `flex items-center w-fit gap-2 px-4 py-2 font-bold uppercase justify-center transition ${getClass(variant)} ${className}`
)

let isAnchor = $derived("href" in props)

function getAnchorProps(props: AnchorProps): Omit<AnchorProps, keyof CommonProps> {
  const { children, class: _, loading, variant, ...anchorProps } = props
  return anchorProps
}

function getButtonProps(props: ButtonProps): Omit<ButtonProps, keyof CommonProps> {
  const { children, class: _, loading, variant, ...buttonProps } = props
  return buttonProps
}
</script>

{#if isAnchor}
  <a class={combinedClasses} {...getAnchorProps(props as AnchorProps)}>
    {@render children()}
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </a>
{:else}
  <button class={combinedClasses} {...getButtonProps(props as ButtonProps)}>
    {@render children()}
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </button>
{/if}