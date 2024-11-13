<script lang="ts">
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
</script>

<h2 class="font-bold font-supermolot text-xl mt-2 mb-1">{@render children?.()}</h2>
