<script lang="ts">
import { cn } from "$lib/utils"
import type { Snippet } from "svelte"
import type { HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLHeadingElement> & {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()

const classes = cn("font-bold text-3xl", className)
</script>

<h2
  class={classes}
  {...rest}
>
  {@render children()}
</h2>
