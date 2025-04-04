<script lang="ts">
import Truncate from "$lib/components/ui/Truncate.svelte"

interface Props {
  hash: string
}

const { hash }: Props = $props()
</script>

<div class="font-mono text-xs break-all">
  <Truncate showCopy={false} value={hash} maxLength={12} />
</div>
