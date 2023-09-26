<slot />

<!-- this is actually a layout for the blog posts !-->

<style lang="postcss">
	/* \u003A is a colon */
	:global(.blog-post-container > p) {
		@apply sm:text-xl mb-4;
	}

	:global(.blog-post-container > h2) {
		@apply sm:text-3xl sm:mt-4 mb-0 font-semibold mb-4;
	}
</style>
