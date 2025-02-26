<script lang="ts">
import type { Snippet } from "svelte"

// Card.svelte
type Props = {
  children: Snippet
  class?: string
}

// Define props with default
const { children, class: className = "" }: Props = $props()
</script>

<div class="flex felx-col gap-4 p-4 {className}">
    {@render children()}
</div>
