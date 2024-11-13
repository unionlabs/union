<script lang="ts">
import "temporal-polyfill/global"
	interface Props {
		children?: import('svelte').Snippet;
	}

	let { children }: Props = $props();
</script>

{@render children?.()}
