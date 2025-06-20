<script lang="ts">
import SharpPlayCircleIcon from "$lib/components/icons/SharpPlayCircleIcon.svelte"
import { Data, Option } from "effect"

interface Props {
  title: string
  videoUrl: string
  description?: string
  thumbnailUrl?: string
  class?: string
}

let { title, videoUrl, description, thumbnailUrl, class: className = "" }: Props = $props()

// Define proper state enum using Effect's Data module
type ViewState = Data.TaggedEnum<{
  Collapsed: {}
  Expanded: {}
  Minimized: {}
}>

const { Collapsed, Expanded, Minimized } = Data.taggedEnum<ViewState>()

let viewState = $state<ViewState>(Collapsed())

// Extract YouTube video ID from various YouTube URL formats
function getYouTubeVideoId(url: string): Option.Option<string> {
  const patterns = [
    /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)([^&\n?#]+)/,
    /youtube\.com\/v\/([^&\n?#]+)/,
  ]

  for (const pattern of patterns) {
    const match = url.match(pattern)
    if (match) {
      return Option.some(match[1])
    }
  }

  return Option.none()
}

const videoId = $derived(getYouTubeVideoId(videoUrl))
const embedUrl = $derived(
  Option.getOrElse(
    Option.map(videoId, (id) => `https://www.youtube.com/embed/${id}`),
    () => videoUrl,
  ),
)
const autoThumbnailUrl = $derived(
  Option.map(videoId, (id) => `https://img.youtube.com/vi/${id}/mqdefault.jpg`),
)
const finalThumbnailUrl = $derived(
  Option.fromNullable(thumbnailUrl).pipe(
    Option.orElse(() => autoThumbnailUrl),
  ),
)

function toggleExpanded() {
  viewState = viewState._tag === "Expanded" ? Collapsed() : Expanded()
}

function minimize() {
  viewState = Minimized()
}

function restore() {
  viewState = Collapsed()
}
</script>

{#if viewState._tag === "Minimized"}
  <!-- Minimized state - small button -->
  <button
    onclick={restore}
    class="rounded-lg bg-zinc-925 border border-zinc-800 shadow-xl p-3 hover:bg-zinc-900 transition-all duration-200 text-white {className}"
  >
    <SharpPlayCircleIcon class="size-5" />
  </button>
{:else}
  <!-- Normal popup state -->
  <div class="rounded-lg bg-zinc-925 border border-zinc-800 shadow-xl backdrop-blur-sm {className}">
    <!-- Header with controls -->
    <div class="flex items-center justify-between p-3 border-b border-zinc-800">
      <div class="flex items-center gap-2">
        <SharpPlayCircleIcon class="size-4 text-zinc-300" />
        <span class="text-sm font-medium text-white">Tutorial</span>
      </div>
      <div class="flex items-center gap-1">
        <button
          onclick={toggleExpanded}
          class="rounded p-1 hover:bg-zinc-800 text-white text-sm w-6 h-6 flex items-center justify-center transition-colors"
          title={viewState._tag === "Expanded" ? "Collapse" : "Expand"}
        >
          {viewState._tag === "Expanded" ? "−" : "+"}
        </button>
        <button
          onclick={minimize}
          class="rounded p-1 hover:bg-zinc-800 text-white text-sm w-6 h-6 flex items-center justify-center transition-colors"
          title="Minimize"
        >
          ×
        </button>
      </div>
    </div>

    {#if viewState._tag === "Expanded"}
      <!-- Expanded content -->
      <div class="p-3">
        <h4 class="text-sm font-medium mb-2 text-white">{title}</h4>
        {#if description}
          <p class="text-xs text-zinc-400 mb-3">{description}</p>
        {/if}

        <div class="relative aspect-video bg-zinc-800 mb-3 overflow-hidden rounded-lg">
          <iframe
            src={embedUrl}
            title={title}
            class="absolute inset-0 w-full h-full"
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
            allowfullscreen
          ></iframe>
        </div>

        <a
          href={videoUrl}
          target="_blank"
          rel="noopener noreferrer"
          class="text-xs text-zinc-300 hover:text-white transition-colors"
        >
          Watch on YouTube →
        </a>
      </div>
    {:else}
      <!-- Collapsed content - just thumbnail and play button -->
      <div class="p-3">
        <button
          onclick={toggleExpanded}
          class="relative w-full aspect-video bg-zinc-800 flex items-center justify-center hover:bg-zinc-700 transition-colors overflow-hidden group rounded-lg"
        >
          {#if Option.isSome(finalThumbnailUrl)}
            <img
              src={finalThumbnailUrl.value}
              alt={title}
              class="absolute inset-0 w-full h-full object-cover group-hover:scale-105 transition-transform duration-200"
              loading="lazy"
            />
          {/if}
          <div
            class="relative bg-black bg-opacity-70 text-white px-3 py-2 text-xs font-medium backdrop-blur-sm border border-white/20 rounded"
          >
            ▶ Play Tutorial
          </div>
        </button>
      </div>
    {/if}
  </div>
{/if}
