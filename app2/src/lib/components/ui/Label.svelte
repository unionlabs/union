<script lang="ts">
import type { Snippet } from "svelte"
import type { HTMLLabelAttributes } from "svelte/elements"

interface Props extends HTMLLabelAttributes {
  children: Snippet
  class?: string
}

const { children, class: className = "", ...rest }: Props = $props()
</script>

<label
  class="uppercase text-zinc-500 text-xs font-semibold block {className}"
  {...rest}
>{@render children()}</label>
