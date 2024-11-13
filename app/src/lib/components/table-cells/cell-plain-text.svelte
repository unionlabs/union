<script lang="ts">
	interface Props {
		value: string;
		[key: string]: any
	}

	let { value, ...rest }: Props = $props();
</script>

<p {...rest}>{value}</p>
