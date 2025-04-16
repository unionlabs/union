<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"
import ExternalLinkIcon from "$lib/components/icons/ExternalLinkIcon.svelte"

type Props = HTMLAttributes<HTMLAnchorElement> & {
  children: Snippet
  class?: string
  href: string
  external?: boolean
}

const { children, class: className = "", href, external = true, ...rest }: Props = $props()

const classes = cn(
  // Base styles
  "underline text-babylon-orange hover:text-orange-300 transition-colors inline-flex items-center gap-1",
  // Additional classes passed as props
  className
)

// Add external link attributes if needed
const externalAttrs = external
  ? {
      target: "_blank",
      rel: "noopener noreferrer"
    }
  : {}
</script>

<a
  {href}
  class={classes}
  {...externalAttrs}
  {...rest}
>
  <span>{@render children()}</span>
  {#if external}
    <ExternalLinkIcon size={14} class="ml-0.5" />
  {/if}
</a>
