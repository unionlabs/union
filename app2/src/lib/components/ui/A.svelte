<script lang="ts">
import { cn } from "$lib/utils"
import type { HTMLAttributes } from "svelte/elements"
import type { Snippet } from "svelte"

type Props = HTMLAttributes<HTMLAnchorElement> & {
  children: Snippet
  class?: string
  href: string
  external?: boolean
}

const {
  children,
  class: className = "",
  href,
  external = true,
  ...rest
}: Props = $props()

const classes = cn(
  // Base styles
  "text-sky-400 hover:text-sky-300 transition-colors",
  // Additional classes passed as props
  className
)

// Add external link attributes if needed
const externalAttrs = external ? {
  target: "_blank",
  rel: "noopener noreferrer"
} : {}
</script>

<a
  {href}
  class={classes}
  {...externalAttrs}
  {...rest}
>
  {@render children()}
</a>
