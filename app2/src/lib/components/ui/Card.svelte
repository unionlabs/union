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

<div class="rounded border dark:border-zinc-700 dark:bg-zinc-900 shadow-sm {className}">
  {@render children()}
</div>
