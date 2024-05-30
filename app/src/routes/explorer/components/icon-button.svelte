<script lang="ts">
import type { SvelteComponent } from "svelte"
import Button from "$lib/components/ui/button/button.svelte"

export let icon: { class: string; component: typeof SvelteComponent }
</script>

<Button {...$$restProps}>
  <svelte:component this={icon.component} class={icon.class} />
</Button>
