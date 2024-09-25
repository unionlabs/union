<script lang="ts">
import { type HTMLAttributes } from "svelte/elements"

type Props = HTMLAttributes<HTMLParagraphElement> & {
  class?: string
}

let { children, class: className = "", ...rest }: Props = $props()
</script>

<p class="w-fit text-neutral-200 {className}" {...rest}>{@render children()}</p>