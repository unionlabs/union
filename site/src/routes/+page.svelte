<script lang="ts">
	import Header from '$lib/Header.svelte';
</script>

<main class="grid-effect-bg flex-1 flex-col flex gap-8 shadow-effect-bg p-4">
	<Header />
	<div class="flex flex-col items-center">
		<div class="main-text">Infrastructure for the</div> 
		<div class="next-century-text">next century</div>
		<div class="tagline">
			Union is a hyper-efficient, zero-knowledge<br>
			interoperability layer that connects<br> 
			appchains, layer 1, and layer 2 networks.
		</div> 
		<div>
			<button class=" bg-accent px-4 py-2 mt-8 text-lg md:text-2xl font-bold text-black">EXPLORE -&gt;</button>
		</div>
	</div>

	<div class="flex flex-col items-center mt-40">
		<h2 class="text-3xl font-bold">Backed by industry experts</h2>

		<div class="flex flex-col md:flex-row gap-6 items-center">
			<img class="block w-[180px] lg:w-[200px]" src="/images/investors/galileo.svg" alt="Galileo"/>
			<img class="block w-[140px] lg:w-[200px]"src="/images/investors/nascent.svg" alt="Nascent"/>
			<img class="block w-[180px] lg:w-[200px]"src="/images/investors/tioga.png" alt="Tioga"/>
		</div>

		<div class="flex flex-col md:flex-row gap-6 items-center ">
			<img class="block w-[200px] lg:w-[220px]"src="/images/investors/semantic.png" alt="Semantic"/>
			<img class="block w-[200px] lg:w-[220px]"src="/images/investors/lightshift.png" alt="Lightshift"/>
		</div>
		
	</div>
</main>


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

	.main-text {
		margin-top: 2rem;
		max-width: 72rem; 
		font-size: 26px;
		line-height: 2.5rem; 
		font-weight: 900; 
		text-align: center; 

		@media (min-width: 340px) {
			font-size: 29px;
		}

		@media (min-width: 380px) {
			font-size: 34px;
		}

		@media (min-width: 600px) { 
			font-size: 52px;
		}

		@media (min-width: 768px) {
			font-size: 65px;
		}

		@media (min-width: 1024px) { 
			font-size: 90px;
			margin-top: 120px;
		 }
	}

	.next-century-text {
		margin-top: -1rem; 
		font-style: italic; 
		font-size: 40px;
		font-weight: 900; 
		text-align: center; 
		color: theme(colors.accent);
		/* text-shadow: 2px 2px #c63f73; */

		@media (min-width: 340px) {
			font-size: 42px;
		}

		@media (min-width: 380px) {
			margin-top: -20px;
			font-size: 44px;
		}

		@media (min-width: 600px) { 
		  margin-top: -20px; 
		  font-size: 60px;
		}

		@media (min-width: 768px) {
			font-size: 70px;
		}

		@media (min-width: 1024px) { 
		  margin-top: 4px; 
		  font-size: 110px;
		  line-height: 1; 
		}
	}

	.tagline {
		font-family: theme(fontFamily.mono);
		font-size: 9.5px;
		margin-top: 8px;

		@media (min-width: 355px) {
			font-size: 11px;
		}
		@media (min-width: 380px) {
			font-size: 12px;
		}

		@media (min-width: 600px) { 
			font-size: 16px;
			margin-top: 12px;
		}

		@media (min-width: 1024px) { 
			font-size: 24px;
			margin-top: 16px;
		}
		
	}
</style>
