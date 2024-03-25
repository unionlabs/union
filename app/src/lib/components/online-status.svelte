<script lang="ts">
import clsx from "clsx"
import { onlineStatus } from "$/lib/online-status.ts"
import { flyAndScale } from "$lib/utilities/shadcn.ts"
import { blockHeightQuery } from "$lib/queries/block.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Tooltip from "$lib/components/ui/tooltip/index.ts"

$: blockHeightStore = blockHeightQuery()
$: blockHeight = $blockHeightStore.data
</script>

<Tooltip.Root>
  <Tooltip.Trigger let:builder class="ml-2 mb-0.5 my-a fixed bottom-0 z-50 w-18">
    <Button
      variant="outline"
      builders={[builder]}
      class={clsx([
        'rounded-full size-3 p-0 animate-pulse -mb-0.5',
        onlineStatus ? 'bg-green-500 hover:bg-green-500' : 'bg-red-500 hover:bg-red-500',
      ])}
    ></Button>
    <span class="my-auto mb-2 text-xs text-foreground/60">{blockHeight}</span>
  </Tooltip.Trigger>
  <Tooltip.Content
    sideOffset={8}
    transition={flyAndScale}
    class="top-0 fixed mt-7 -ml-0.5 p-3 z-40"
    transitionConfig={{ y: 8, duration: 150 }}
  >
    <div
      class={clsx([
        onlineStatus ? 'border-cyan-300/30' : 'border-rose-500/30',
        'flex items-center justify-center border bg-background p-3 text-sm font-medium border-solid rounded-md',
      ])}
    >
      App is online
    </div>
  </Tooltip.Content>
</Tooltip.Root>
