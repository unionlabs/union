<script>
	export let title;
	export let date;
	export let author;
	export let preview;
	export let published
</script>


<div class="bg-black flex flex-1 justify-center">
<main
	class="blog-post-container sm:text-xl p-4 sm:mt-4  justify-center max-w-2xl justify-center self-center"
>
	{#if published}
	<h1 class="text-3xl sm:text-5xl font-bold mb-0">{title}</h1>
	<div class="font-mono mb-2 mt-1 sm:text-lg"
		>{date} - <a class="text-accent" href={`https://x.com/${author}`}>@{author}</a></div
	>
	<slot />
	{/if}
</main>
</div>
