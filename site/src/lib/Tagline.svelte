<script lang="ts">
	import Typewriter from 'svelte-typewriter';
</script>

<h2
	class="text-lg md:text-3xl font-semibold h-[36px] mt-[15px] md:mt-[35px] mb-[12px] md:mb-[24px] tagline flex gap-1 md:gap-2"
>
	<span>Infrastructure for</span>
	<Typewriter mode="loop" delay={5000} showCursorOnDelay={true} cursor={false}>
		<span class="text-accent font-bold flex">DeFi</span>
		<span class="text-accent font-bold flex">Shared Security</span>
		<span class="text-accent font-bold flex">NFTs</span>
		<span class="text-accent font-bold flex">Governance</span>
		<span class="text-accent font-bold flex">Horizontal Scalability</span>
		<span class="text-accent font-bold flex">Mass Adoption</span>
		<span class="text-accent font-bold flex">The Next Century</span>
	</Typewriter>
</h2>

<style>
	@keyframes fadeIn {
		0% {
			opacity: 0;
		}
		100% {
			opacity: 1;
		}
	}

	.tagline {
		animation-duration: 1.1s;
		animation-delay: 5.3s;
		animation-name: fadeIn;
		animation-timing-function: ease-in;
		animation-fill-mode: both;
	}
</style>
