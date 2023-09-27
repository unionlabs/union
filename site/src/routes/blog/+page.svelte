<script lang="ts">
	type PostMeta = { metadata: { title: string, date: string, author: string, preview: string }};
	const posts: Record<string, PostMeta> = import.meta.globEager('../**/*.md');
</script>


<main class="flex flex-1 justify-center sm:mt-4 ">
	<div class="flex flex-col gap-4 max-w-2xl">
		{#each Object.entries(posts) as [path, {metadata}] }
			<section class="p-4">
				<h1 class="text-3xl sm:text-5xl font-bold mb-0">{metadata.title}</h1>
				<div class="font-mono mb-2 mt-1 sm:text-lg">{metadata.date} - <a class="text-accent" href={`https://x.com/${metadata.author}`}>{metadata.author}</a></div>
				<p class="sm:text-xl">{metadata.preview}</p>
				<a class="block mt-2 font-mono text-lg" href="/blog/the-journey-so-far">READ FULL POST -&gt;</a>
			</section>
		{/each}
	</div>
</main>
