<script>
	import '../app.css';
	import Header from '$lib/Header.svelte';
	import Footer from '$lib/Footer.svelte';
</script>

<svelte:head>
	<title>Union</title>
</svelte:head>

<div class="flex flex-col min-h-screen">
	<Header />
	<slot />
	<Footer />
</div>

<style lang="postcss">
	:global(body) {
		background-color: theme(colors.black);
		color: theme(colors.primary);
	}
</style>
