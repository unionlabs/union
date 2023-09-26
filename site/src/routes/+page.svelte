<script lang="ts">
	import Logo from '$lib/Logo.svelte';
	import ButtonA from '$lib/ButtonA.svelte';
	import Tagline from '$lib/Tagline.svelte';
</script>

<div class=" max-w-screen relative flex items-center justify-center overflow-hidden">
	<div class="flex flex-col justify-center items-center">
		<Logo />
		<Tagline />
		<div class="flex flex-row md:flex-row gap-[16px] md:gap-[32px]">
			<div class="buttonFly"><ButtonA href="https://docs.union.build">Docs</ButtonA></div>
			<div class="buttonFly"><ButtonA href="https://github.com/unionlabs">GitHub</ButtonA></div>
			<div class="buttonFly"><ButtonA href="https://x.com/union_build">X</ButtonA></div>
		</div>
		<div class="h-[33px] md:h-[67px]" />
		<!--
    <p class="text-xl font-semibold">Infrastructure for <span class="text-accent font-bold">shared security</span>.<p>
    !-->
	</div>
</div>

<style lang="postcss">
	@keyframes fadeIn {
		0% {
			opacity: 0;
		}
		100% {
			opacity: 1;
		}
	}

	.buttonFly {
		animation-duration: 1.1s;
		animation-delay: 5.3s;
		animation-name: fadeIn;
		animation-timing-function: ease-in;
		animation-fill-mode: both;
	}
</style>
