<script>
	export let title;
	export let date;
	export let author;
</script>

<main
	class="blog-post-container flex flex-col flex-1 p-4 justify-center max-w-2xl justify-center self-center"
>
	<h1 class="text-3xl sm:text-5xl font-bold">{title}</h1>
	<span class="font-mono mb-2 mt-1 sm:text-lg"
		>{date} - <a class="text-accent" href={`https://x.com/${author}`}>{author}</a></span
	>

	<slot />
</main>

<!--
<style lang="postcss">
	:global(.blog-post-container > h1 + p) {
		@apply font-mono mb-2 mt-1 sm:text-lg;
	}
	:global(.blog-post-container > h2 + span > a) {
		@apply text-accent;
	}

	:global(.blog-post-container > p) {
		@apply sm:text-xl mb-4;
	}
</style>
!-->
