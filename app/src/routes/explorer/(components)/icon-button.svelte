<script lang="ts">
import type { SvelteComponent } from "svelte"
import Button from "$lib/components/ui/button/button.svelte"

  interface Props {
    icon: { class: string; component: typeof SvelteComponent };
    [key: string]: any
  }

  let { icon, ...rest }: Props = $props();
</script>

<Button {...rest}>
  <icon.component class={icon.class} />
</Button>
