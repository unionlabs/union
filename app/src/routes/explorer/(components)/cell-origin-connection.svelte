<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

export let value: {
  chain_display_name: string
  chain_id: string
  connection_id: string
  client_id: string
}
</script>

<div  class={cn("flex flex-col ")} {...$$restProps}>
  <div class="font-bold">{value.chain_display_name}</div>
  <div>{value.chain_id}</div>
  <div>{value.connection_id}</div>
  <div>{value.client_id}</div>
</div>
