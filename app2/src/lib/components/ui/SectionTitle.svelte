<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utils"
import type { Snippet } from "svelte"

type Props = HTMLAttributes<HTMLHeadingElement> & {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()

const classes = cn("font-bold text-4xl", className)
</script>

<h2 class={classes} {...rest}>
  {@render children()}
</h2>
