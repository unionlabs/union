<script lang="ts">
import Chevron from "./chevron.svelte"
import { Button } from "$lib/components/ui/button/index.ts"

interface Props {
  dialogOpen: boolean
  children?: import("svelte").Snippet
  [key: string]: any
}

let { dialogOpen = $bindable(), children, ...rest }: Props = $props()
</script>

<Button
  {...rest}
  variant="outline"
  data-transfer-from-chain=""
  on:click={() => (dialogOpen = !dialogOpen)}
  class="flex flex-row items-center w-full"
>
  <div class="flex items-center space-x-1.5 flex-1">
    <div class="flex flex-col items-start">
      <div class="font-bold text-md mr-auto w-full text-left">
        {@render children?.()}
      </div>
    </div>
  </div>
  <Chevron/>
</Button>
