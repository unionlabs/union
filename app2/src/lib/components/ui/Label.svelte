<script lang="ts">
import type { Snippet } from "svelte"

// Card.svelte
type Props = {
  children: Snippet
  class?: string
}

// Define props with default
const { children, class: className = "", ...rest }: Props = $props()
</script>
<label
  class="uppercase text-zinc-500 text-xs font-semibold block {className}"
  {...rest}
>{@render children()}</label>
