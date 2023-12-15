<script lang="ts">
    import Footer from '$lib/Footer.svelte';
	import Header from '$lib/Header.svelte';
</script>

<main class="flex flex-col flex-1 gap-[100px] md:gap-[200px] my-[50px] md:my-[100px]">
	<div class="flex flex-col items-center">
		<div class="main-text">Infrastructure for the</div> 
		<div class="next-century-text animate-character">next century</div>
		<div class="tagline">
			Union is a hyper-efficient, zero-knowledge interoperability layer<br> 
			that connects Appchains, Layer 1, and Layer 2 networks.
		</div> 
		<div>
			<a class="skew-button" href="https://docs.union.build"><span>EXPLORE THE DOCS -&gt;</span></a>
		</div>
	</div>

	<section>
		<img class="w-[300px] md:w-[500px] absolute left-[50%] -ml-[150px] md:-ml-[250px] z-10 -mt-[50px] md:-mt-[100px] eth-logo" src="/images/ethereum.svg" alt="Ethereum"/>
		<div class="flex flex-col items-center z-20 relative">
			<h2 class="text-4xl md:text-6xl font-bold text-center">ETH &lt;-&gt; IBC</h2>
			<h3 class="text-[16px] md:text-[27px] font-bold text-center text-gray-200">Union's flagship product</h3>
			<p class="md:text-2xl text-center mt-10">Built using Zero-Knowledge Cryptography and BLS Signatures, a</p>
			<div class="text-center trustless-connection text-2xl md:text-5xl text-accent font-black italic animate-character animate-character-delay">Trustless Ethereum IBC Connection</div>
			<p class="text-center md:text-2xl">Live between Union's Testnet and Ethereum's Testnet.</p>
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
			<a class="skew-button" href="/blog/seed-raise"><span>READ THE ANNOUNCEMENT -&gt;</span></a>
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

		padding: 2px 8px;
	
		font-style: italic; 
		font-size: 40px;
		font-weight: 900; 
		text-align: center; 
		color: theme(colors.accent);
		/* text-shadow: 2px 2px #c63f73; */
		margin: -24px -8px;

		@media (min-width: 340px) {
			font-size: 42px;
		}

		@media (min-width: 380px) {
			font-size: 44px;
		}

		@media (min-width: 600px) { 
		  font-size: 60px;
		}

		@media (min-width: 768px) {
			font-size: 70px;
		}

		@media (min-width: 1024px) { 
		  font-size: 110px;
		  line-height: 1; 
			padding: 8px 16px;
			margin: -8px 0;
		}
	}

	.tagline {
		text-align: center;
		font-size: 10px;
		margin-top: 8px;

		@media (min-width: 360px) {
			font-size: 12px;
		}
		@media (min-width: 425px) {
			font-size: 14px;
		}

		@media (min-width: 600px) { 
			font-size: 16px;
		}

		@media (min-width: 768px) {
			font-size: 24px;
		}
	}

	@keyframes textclip {
	  to {
	    background-position: 200% center;
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
		font-size: 18px;
		
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

	.trustless-connection {
		padding: 0 16px;
		margin: 0 -16px;
	}

	.animate-character {
	  -webkit-background-clip: text;
	  -webkit-text-fill-color: transparent;
	  animation: textclip 2s linear infinite;
		animation-direction: reverse;

	  background-image: linear-gradient(
	    100deg,
	    theme(colors.accent) 0%,
	    theme(colors.accent) 40%,
	    theme(colors.accentBright) 50%,
	    theme(colors.accent) 60%,
	    theme(colors.accent) 100%
	  );
	  background-size: 100% 100%;
	  background-size: 200% 100%;
		background-clip: text;
	}


	.animate-character-delay {
		animation-delay: -0.2s;
	}
</style>
