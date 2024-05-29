<script lang="ts">
import { onlineStatus } from "$lib/online-status.ts"
import { blockHeightQuery } from "$lib/queries/block.ts"
import { cn, flyAndScale } from "$lib/utilities/shadcn.ts"
import { Button } from "$lib/components/ui/button/index.ts"
import * as Tooltip from "$lib/components/ui/tooltip/index.ts"

/**
 * TODO:
 * - Split checks into two: app status and block height status
 */

$: blockHeightStore = blockHeightQuery()
$: blockHeight = $blockHeightStore.data
</script>

<Tooltip.Root>
  <Tooltip.Trigger let:builder class="ml-0.75 mb-16 lg:mb-0.5 my-a fixed bottom-0 z-50 w-18">
    <Button
      variant="outline"
      builders={[builder]}
      class={cn(
        'rounded-full size-2.5 p-0 animate-pulse -mb-0.25',
        onlineStatus ? 'bg-green-500 hover:bg-green-500' : 'bg-red-500 hover:bg-red-500',
      )}
    ></Button>
    <span class="my-auto mb-2 text-xs text-foreground/90">{blockHeight}</span>
  </Tooltip.Trigger>
  <Tooltip.Content
    sideOffset={8}
    transition={flyAndScale}
    class="top-0 fixed mt-5 -ml-3 p-3 z-40"
    transitionConfig={{ y: 8, duration: 150 }}
  >
    <div
      class={cn(
        onlineStatus ? 'border-cyan-300/30' : 'border-rose-500/30',
        'flex flex-col items-start justify-center border bg-background p-3 text-sm font-medium border-solid rounded-md',
      )}
    >
      <span>App is online</span>
      <span>Height {blockHeight}</span>
    </div>
  </Tooltip.Content>
</Tooltip.Root>
