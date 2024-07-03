  <script lang="ts">
import { deviceWidth } from "$lib/utilities/device.ts"
import { cn } from "$lib/utilities/shadcn.ts"

export let label: string
export let value: number | string
</script>
  
  <div class={cn("uppercase  flex px-6 py-4 border-none", $deviceWidth >= 888 ? "w-fit" : "w-full")}>
    <div class="flex flex-col divide-y">
      <h3 class="text-sm pb-2">{label}</h3>
      <div class="text-xl pt-2 font-bold">{value}</div>
    </div>
    <slot />
  </div>