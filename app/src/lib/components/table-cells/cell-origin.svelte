<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

interface Props {
  value: {
    name: string
    chain: unknown
    connection: unknown
    channel: unknown
  }
  [key: string]: any
}

let { value, ...rest }: Props = $props()
</script>

<div  class={cn("flex flex-col ")} {...rest}>
  <div class="font-bold">{value.name}</div>
  <div>{value.chain}</div>
  <div>{value.connection}</div>
  <div>{value.channel}</div>
</div>
