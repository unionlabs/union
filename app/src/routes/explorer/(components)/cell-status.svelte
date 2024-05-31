<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

export let value: string

const goodValue = (v: string) => v === "HEALTHY" || v === "CONFIRM"
</script>

<div class={cn(goodValue(value) ? 'bg-green-500 dark:bg-green-800' : 'bg-red-500 dark:bg-red-800', 'font-bold text-white inline px-2 py-1 rounded')}  {...$$restProps}>{value}</div>
