
<slot />

<!-- this is actually a layout for the blog posts !-->

<style lang="postcss">
	/* \u003A is a colon */
	:global(.blog-post-container > p) {
		@apply mb-4;
	}

	:global(.blog-post-container > h2) {
		@apply text-2xl sm:text-3xl mt-4 sm:mt-8 mb-0 font-bold;
	}

	:global(.blog-post-container > h3) {
		@apply text-xl sm:text-2xl mt-4 sm:mt-8 mb-0 font-bold;
	}

	:global(.blog-post-container ol) {
		@apply list-decimal list-inside mb-4;
	}

	:global(.blog-post-container ul) {
		@apply list-disc list-inside mb-4;
	}

	:global(.blog-post-container a) {
		@apply text-accent font-semibold;
	}

	:global(.blog-post-container code) {
		@apply font-jetbrains;
		color: #c0c0c5;
		background: #0f0f10;
		padding: 3px 7px 3px;
  	border-radius: 7px;
	}
</style>
