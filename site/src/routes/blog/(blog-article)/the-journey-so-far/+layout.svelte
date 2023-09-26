<main
	class="blog-post-container flex flex-col flex-1 p-4 justify-center max-w-2xl justify-center self-center"
>
	<slot />
</main>

<style lang="postcss">
	:global(.blog-post-container > h2) {
		@apply text-5xl font-bold;
	}

	:global(.blog-post-container > .subtitle) {
		@apply font-mono mb-2 text-lg;
	}
</style>
