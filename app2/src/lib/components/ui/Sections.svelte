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

<div class="flex flex-col gap-4 p-4 md:gap-6 md:p-8 lg:px-16 xl:px-32 {className}">
  {@render children()}
</div>
