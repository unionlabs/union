<script lang="ts">
    import Footer from '$lib/Footer.svelte';
	import Header from '$lib/Header.svelte';
</script>

<main class="flex flex-col flex-1 gap-[100px] md:gap-[200px] my-[50px] md:my-[100px]">
	<div class="flex flex-col items-center">
		<div class="main-text">Infrastructure for the</div> 
		<div class="next-century-text">next century</div>
		<div class="tagline">
			Union is a hyper-efficient, zero-knowledge<br>
			interoperability layer that connects<br> 
			appchains, layer 1, and layer 2 networks.
		</div> 
		<div>
			<a class="skew-button" href="https://docs.union.build"><span>EXPLORE THE DOCS -&gt;</span></a>
		</div>
	</div>



	<section>
		<img class="w-[300px] md:w-[500px] absolute left-[50%] -ml-[150px] md:-ml-[250px] z-10 -mt-[50px] md:-mt-[100px] eth-logo" src="/images/ethereum.svg" alt="Ethereum"/>
		<div class="flex flex-col items-center z-20 relative">
			<h2 class="text-4xl md:text-6xl font-bold text-center">ETH &lt;-&gt; IBC</h2>
			<h3 class="text-[16px] md:text-[27px] font-bold text-center text-gray-300">Union's flagship product</h3>
			<p class="md:text-2xl mt-4 text-center mt-10">Built using Zero-Knowledge Cryptography and BLS Signatures, a</p>
			<div class="text-center text-2xl md:text-5xl text-accent font-black italic">Trustless Ethereum IBC Connection</div>
			<p class="text-center md:text-2xl">Live between Union' Testnet and Ethereum's Testnet.</p>
			<div class="mt-4">
				<a class="skew-button" href="/blog/ics20-transfers-to-ethereum/"><span>TRY IT OUT -&gt;</span></a>
			</div>
		</div>
	</section>

	<section class="flex flex-col items-center">
		<h2 class="text-xl md:text-3xl font-bold">Backed by industry experts</h2>

		<div class="flex flex-col md:flex-row gap-[24px] mt-[24px] items-center">
			<img class="block w-[200px]" src="/images/investors/galileo.svg" alt="Galileo"/>
			<img class="block w-[60px] mx-8"src="/images/investors/nascent.svg" alt="Nascent"/>
			<img class="block w-[200px]"src="/images/investors/tioga.png" alt="Tioga"/>
		</div>

		<div class="flex flex-col md:flex-row gap-[24px] mt-[24px] items-center ">
			<img class="block w-[220px]"src="/images/investors/semantic.png" alt="Semantic"/>
			<img class="block w-[220px]"src="/images/investors/lightshift.png" alt="Lightshift"/>
		</div>
		<div class="mt-4">
			<a class="skew-button" href="/blog"><span>READ THE ANNOUNCEMENT -&gt;</span></a>
		</div>
	</section>
</main>	


<style lang="postcss">

	.main-text {
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

	.skew-button {
	  font-size: 12px;
	  margin-top: 16px;

	  @media(min-width: 600px) {
	  	font-size: 16px;
	  }

	  @media(min-width: 1024px) {
		margin-top: 24px;
;		font-size: 18px;
		
	  }
	  --skew: 16deg;
	  color: theme(colors.black);
	  background: theme(colors.accent);
	  font-weight: 700;
	  text-align: center;
	  text-decoration: none;
	  border: 0;
	  outline: 0;
	  padding: 8px 24px;
	  display: inline-block;
	  transform: skew(calc(var(--skew) * -1));
	}

    /* Un-skew the text */
	.skew-button > span {
	  display: inline-block;
	  transform: skew(var(--skew));
	}

	.eth-logo {
		opacity: 0.2;

	}
</style>
