<script lang="ts">
	type PostMeta = { metadata: { title: string, date: string, author: string, preview: string }};
	const posts: Record<string, PostMeta> = import.meta.globEager('../**/*.md');
	const slugRegex = /\/([^/]+)\//;

	function extractSlug(path: string): string {
		const matches= slugRegex.exec(path)
		if (matches === null) {
			return '/404';
		};
		return `/blog/${matches[1]}`;
	} 
</script>


<main class="flex flex-1 justify-center sm:mt-4 ">
	<div class="flex flex-col gap-4 max-w-2xl">
		{#each Object.entries(posts) as [path, {metadata}] }
			<a class="block p-4" href={extractSlug(path)}>
				<h1 class="text-3xl sm:text-5xl font-bold mb-0">{metadata.title}</h1>
				<div class="font-mono mb-2 mt-1 sm:text-lg">{metadata.date} - <a class="text-accent" href={`https://x.com/${metadata.author}`}>{metadata.author}</a></div>
				<p class="sm:text-xl">{metadata.preview}</p>
				<p class="block mt-2 font-mono text-lg">READ FULL POST -&gt;</p>
			</a>
		{/each}
	</div>
</main>
