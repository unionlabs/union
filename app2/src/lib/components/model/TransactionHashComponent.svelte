<script lang="ts">
import Truncate from "$lib/components/ui/Truncate.svelte"

interface Props {
  hash: string
}

const { hash }: Props = $props()
</script>

<div class="font-mono text-sm break-all">
  <Truncate value={hash} maxLength={12} />
</div>
