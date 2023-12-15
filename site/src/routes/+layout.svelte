<script lang="ts">
	import '../app.css';
	import Header from '$lib/Header.svelte';
	import Footer from '$lib/Footer.svelte';
</script>

<svelte:head>
	<title>Union</title>
</svelte:head>

<div class="flex flex-col min-h-screen grid-effect-bg flex-1 gap-8 shadow-effect-bg p-4">
	<Header />
	<slot />
	<Footer />
</div>

<style lang="postcss">
		.grid-effect-bg {
		--grid-size: 4rem;
		@media (min-width: 1024px) { 
			--grid-size: 6rem;
		}
		--grid-line-color: rgba(20, 20, 20, 1);

	  background: black;
	  background-image: 
			radial-gradient(rgba(255, 135, 195, 0.02), rgba(0, 0, 0, 1)),
			linear-gradient(var(--grid-line-color) .1em, transparent .1em), 
			linear-gradient(90deg, var(--grid-line-color) .1em, transparent .1em);
	  background-position: center center;
	  background-size: 
			cover,
			var(--grid-size) var(--grid-size), 
			var(--grid-size) var(--grid-size); 
	}

	:global(body) {
		background-color: theme(colors.black);
		color: theme(colors.primary);
	}
</style>
