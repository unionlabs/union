<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

export let value: string

const goodValue = (v: string) => v === "HEALTHY" || v === "CONFIRM"
</script>

<div class={cn(goodValue(value) ? 'dark:text-[#A0ECFD]' : 'dark:text-[#FD6363]', 'text-black inline px-2 py-1 rounded')}  {...$$restProps}>{value}</div>
