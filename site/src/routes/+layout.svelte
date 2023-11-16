<script>
	import '../app.css';
	import Header from '$lib/Header.svelte';
	import Footer from '$lib/Footer.svelte';
</script>

<svelte:head>
	<title>Union</title>
</svelte:head>

<slot />

<style lang="postcss">
	:global(body) {
		background-color: theme(colors.black);
		color: theme(colors.primary);
	}
</style>
