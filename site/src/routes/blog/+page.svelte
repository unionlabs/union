<script lang="ts">
	type PostMeta = { metadata: { title: string, date: string, author: string, preview: string }};
	const posts: Record<string, PostMeta> = import.meta.globEager('../**/*.md');
</script>


<main class="flex flex-1 p-4 justify-center">
	<div class="flex flex-col gap-4 max-w-xl">
		{#each Object.entries(posts) as [path, {metadata}] }
			<section>
				<h2 class="text-5xl font-bold">{metadata.title}</h2>
				<span class="font-mono mb-2 mt-1 sm:text-lg"
					>{metadata.date} - <a class="text-accent" href={`https://x.com/${metadata.author}`}>{metadata.author}</a></span
				>
				<p class="text-lg">{metadata.preview}</p>
				<a class="font-mono text-lg" href="/blog/the-journey-so-far">READ FULL POST</a>
			</section>
		{/each}
	</div>
</main>
