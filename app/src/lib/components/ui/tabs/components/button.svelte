<script lang="ts">
import Button from "$lib/components/ui/button/button.svelte"

export let value: string
</script>

<Button {...$$restProps}>
  {value}
</Button>
