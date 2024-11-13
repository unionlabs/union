<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"

	interface Props {
		value: string;
		[key: string]: any
	}

	let { value, ...rest }: Props = $props();

const goodValue = (v: string) => v === "HEALTHY" || v === "CONFIRM"
</script>

<div class={cn(goodValue(value) ? 'dark:text-[#A0ECFD]' : 'dark:text-[#FD6363]', 'text-black inline px-2 py-1 rounded')}  {...rest}>{value}</div>
