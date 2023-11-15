<script lang="ts">
	import Logo from '$lib/Logo.svelte';
	import ButtonA from '$lib/ButtonA.svelte';
	import Tagline from '$lib/Tagline.svelte';
	import Header from '$lib/Header.svelte';
</script>

<main class="grid-effect-bg flex-1 flex-col flex gap-8 shadow-effect-bg p-4">
	<Header />
	<div class="flex flex-col items-center">
		<div class="font-black text-8xl text-center max-w-6xl mt-[120px]">Infrastructure for the</div> 
		<div class="italic text-accent text-9xl text-shadow-2xl font-black -mt-10 text-center">next century</div>
		<div class="text-lg md:text-3xl font-mono md:max-w-4xl mt-8">Union is a hyper-efficient, zero-knowledge interoperability layer that connects appchains, layer 1, and layer 2 networks.</div> 
		<div>
			<button class=" bg-accent px-4 py-2 mt-8 text-lg md:text-2xl font-bold text-black">EXPLORE </button>
		</div>
	</div>

	<div class="flex flex-col items-center mt-40">
		<h2 class="text-3xl font-bold">Backed by industry experts</h2>

		<div class="flex flax-col gap-6 items-center">
			<img class="block w-[180px]" src="/images/investors/galileo.svg" alt="Galileo"/>
			<img class="block w-[140px]"src="/images/investors/nascent.svg" alt="Nascent"/>
			<img class="block w-[180px]"src="/images/investors/tioga.png" alt="Tioga"/>
		</div>

		<div class="flex flax-col gap-6 items-center ">
			<img class="block w-[200px]"src="/images/investors/semantic.png" alt="Semantic"/>
			<img class="block w-[200px]"src="/images/investors/lightshift.png" alt="Lightshift"/>
		</div>
		
	</div>
</main>


<style>
	.grid-effect-bg {
		--grid-size: 6rem;
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
</style>
