<script lang="ts">
import PDFJS_WORKER_URL from "pdfjs-dist/build/pdf.worker.min.mjs?url"
import * as R from "remeda"
import { tick } from "svelte"
import { fade } from "svelte/transition"

import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist/types/src/display/api"

type OutlineItem = Awaited<ReturnType<PDFDocumentProxy["getOutline"]>>[number] & {
  items: OutlineItem[]
}

type Engine = {
  open: (url: string | URL) => Promise<PDFDocumentProxy>
  renderPage: (opts: RenderOpts) => Promise<void>
  goToDest: (opts: GoToDestOpts) => Promise<void>
  goToPage: (opts: GoToPageOpts) => void
  destroy: () => void
}

type RenderOpts = {
  doc: PDFDocumentProxy
  pageNumber: number
  canvas: HTMLCanvasElement
  scale: number
  maxPixelRatio?: number
}

type GoToDestOpts = {
  doc: PDFDocumentProxy
  dest: string | any[] | null | undefined
  canvases: HTMLCanvasElement[]
  scrollContainer: HTMLElement
  uiScale: number
}

type GoToPageOpts = {
  page: number
  pageCount: number
  canvases: HTMLCanvasElement[]
}

type Props = {
  src: string
  fileName: string
  invert?: "auto" | boolean | undefined
}
let { src, fileName, invert = false }: Props = $props()

const clamp1 = (n: number, min: number, max: number) => Math.min(Math.max(n, min), max)

const fitWidthScale = (baseWidth: number, containerWidth: number) =>
  Math.max(0.01, containerWidth / Math.max(1, baseWidth))

const cssDims = (viewport: { width: number; height: number }) => ({
  cssWidth: Math.floor(Math.max(1, viewport.width)),
  cssHeight: Math.floor(Math.max(1, viewport.height)),
})

const deviceDims = (cssWidth: number, cssHeight: number, scale: number) => ({
  devWidth: Math.max(1, Math.floor(cssWidth * scale)),
  devHeight: Math.max(1, Math.floor(cssHeight * scale)),
})

// Render pages sequentially without imperative for-loops
const renderSequential = async (
  canvases: HTMLCanvasElement[],
  f: (canvas: HTMLCanvasElement, i: number) => Promise<void>,
) =>
  R.pipe(
    canvases,
    R.map((c, i) => () => f(c, i)),
    R.reduce(
      (p, job) => p.then(job),
      Promise.resolve<void>(undefined),
    ),
  )

// ---------- Engine (pure-ish wrapper around pdf.js) ----------
const createPdfEngine = async (): Promise<Engine> => {
  const { PDFWorker, getDocument } = await import("pdfjs-dist")
  const port = new Worker(PDFJS_WORKER_URL, {
    type: "module",
    name: `pdf-${Math.random().toString(36).slice(2)}`,
  })
  const worker = PDFWorker.create({ port })

  const open = (url: string | URL) => getDocument({ url, worker }).promise

  const renderPage = async (opts: RenderOpts) => {
    const page: PDFPageProxy = await opts.doc.getPage(opts.pageNumber)

    // 1) CSS scale (fit-to-width * user zoom)
    const base = page.getViewport({ scale: 1 })
    const containerWidth = R.defaultTo(
      opts.canvas.parentElement?.clientWidth,
      Math.ceil(base.width),
    )
    const cssScale = Math.max(0.05, fitWidthScale(base.width, containerWidth) * opts.scale)
    const viewport = page.getViewport({ scale: cssScale })

    // 2) Output scale = DPR (clamped), golden-ratio sprinkle for crispness
    const dpr = Math.max(1, Math.min(globalThis.devicePixelRatio || 1, opts.maxPixelRatio ?? 3))
    const outputScale = dpr * 1.618

    // 3) Canvas backing store + CSS size
    const { cssWidth, cssHeight } = cssDims(viewport)
    const { devWidth, devHeight } = deviceDims(cssWidth, cssHeight, outputScale)

    if (opts.canvas.width !== devWidth) {
      opts.canvas.width = devWidth
    }
    if (opts.canvas.height !== devHeight) {
      opts.canvas.height = devHeight
    }

    // CSS size driven by layout; preserve aspect
    Object.assign(opts.canvas.style, {
      maxWidth: `${cssWidth}px`,
      width: "100%",
      height: "auto",
    })

    // 4) Render via canvas handle (no deprecated canvasContext)
    const task = page.render({
      canvas: opts.canvas,
      viewport,
      transform: outputScale !== 1 ? [outputScale, 0, 0, outputScale, 0, 0] : undefined,
      // Explicitly blank the old API for clarity
      canvasContext: undefined as unknown as any,
    })

    await task.promise
  }

  const goToDest = async ({ doc, dest, canvases, scrollContainer, uiScale }: GoToDestOpts) => {
    const resolved = R.isArray(dest) ? dest : await doc.getDestination(dest as any)
    if (R.isNullish(resolved)) {
      return
    }

    const [ref, rawMode, leftRaw, topRaw, z] = resolved
    const pageIndex = typeof ref === "object" ? await doc.getPageIndex(ref) : (ref as number)
    const pageNumber = pageIndex + 1

    const mode = (rawMode?.name as string | undefined) ?? "XYZ"
    const page = await doc.getPage(pageNumber)

    const base = page.getViewport({ scale: 1 })
    const canvas = canvases[pageIndex]
    if (!canvas) {
      return
    }

    const containerWidth = R.defaultTo(canvas.parentElement?.clientWidth, base.width)
    const effectiveScale = Math.max(0.01, uiScale * fitWidthScale(base.width, containerWidth))
    const viewport = page.getViewport({ scale: effectiveScale })

    // Honor absolute zoom when provided
    void z

    const left = (leftRaw ?? 0) as number
    const top = (mode === "XYZ" ? (topRaw ?? base.height) : (topRaw ?? 0)) as number
    const [, vy] = viewport.convertToViewportPoint(left, top)

    const topDelta = canvas.getBoundingClientRect().top
      - scrollContainer.getBoundingClientRect().top
    const offsetInContainer = topDelta + vy

    scrollContainer.scrollTo({
      top: scrollContainer.scrollTop + offsetInContainer - 20,
      behavior: "smooth",
    })
  }

  const goToPage = ({ page, pageCount, canvases }: GoToPageOpts) =>
    R.pipe(
      R.clamp(page, { min: 1, max: pageCount }) - 1,
      (i) => canvases[i],
      (c) => c?.scrollIntoView({ behavior: "smooth", block: "start" }),
    )

  const destroy = () => worker.destroy()

  return { open, renderPage, goToDest, goToPage, destroy }
}

// ---------- State ----------
let pageCount = $state(0)
let canvases = $state<HTMLCanvasElement[]>([])
let scrollContainer = $state<HTMLDivElement | null>(null)
let status = $state<"idle" | "loading" | "ready" | "error">("idle")
let currentPage = $state(1)
let errorMsg = $state("")
let pagesRendered = $state(0)
let doc = $state<PDFDocumentProxy | null>(null)
let inverted = $state<boolean>(false)
let outline = $state<Awaited<ReturnType<PDFDocumentProxy["getOutline"]>>>([])
let engine = $state<Engine | null>(null)
let currentPageInput = $state(1)

// ---------- Derived ----------
const flattenedOutline = $derived.by(() => {
  const flatten = (
    xs: OutlineItem[] | null | undefined,
    indent = 0,
  ): Array<Omit<OutlineItem, "items"> & { indent: number }> =>
    R.pipe(
      xs ?? [],
      R.flatMap(({ items, ...rest }) => [
        { ...rest, indent },
        ...(R.isEmpty(items) ? [] : flatten(items, indent + 1)),
      ]),
    )

  return flatten(outline)
})

// Keep input and derived page in sync
$effect(() => {
  currentPageInput = currentPage
})

// Determine color inversion from browser preferences if allowed
$effect.root(() => {
  const prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
  inverted = invert === "auto" ? prefersDark : invert
})

/**
 * Render lifecycle
 * NOTE: scaled re-rendering is not supported, so we oversample
 */
$effect.root(() => {
  status = "loading"
  errorMsg = ""
  canvases = []
  pagesRendered = 0

  let cancelled = false
  ;(async () => {
    try {
      const e = await createPdfEngine()
      const d = await e.open(src)
      const ol = (await d.getOutline()) ?? []

      if (cancelled) {
        return
      }

      engine = e
      doc = d
      outline = ol
      pageCount = d.numPages

      await tick() // ensure canvases are bound

      await renderSequential(canvases, async (canvas, i) => {
        await e.renderPage({ doc: d, pageNumber: i + 1, canvas, scale: 1 })
        pagesRendered += 1
      })

      if (!cancelled) {
        status = "ready"
      }
    } catch (err: unknown) {
      if (!cancelled) {
        status = "error"
        errorMsg = (err as any)?.message ?? String(err)
      }
    }
  })()

  return () => {
    cancelled = true
    try {
      doc?.destroy().finally(() => engine?.destroy())
    } catch {}
    doc = null
    engine = null
  }
})

/**
 * Current page tracking
 */
$effect(() => {
  if (!scrollContainer || canvases.length === 0) {
    return
  }

  const observer = new IntersectionObserver(
    (entries) => {
      R.pipe(
        entries,
        R.find((e) => e.isIntersecting),
        (hit) => {
          if (!hit) {
            return
          }
          const idx = canvases.findIndex((c) => c === hit.target)
          if (idx !== -1) {
            currentPage = idx + 1
          }
        },
      )
    },
    { root: scrollContainer, rootMargin: "0px 0px -80% 0px", threshold: 0 },
  )

  canvases.forEach((c) => c && observer.observe(c))
  return () => observer.disconnect()
})
</script>

<style>
:host { display: block; }
:global(input[type="number"]::-webkit-inner-spin-button),
:global(input[type="number"]::-webkit-outer-spin-button) {
  -webkit-appearance: none;
  margin: 0;
}
:global(input[type="number"]) {
  appearance: textfield;
  -moz-appearance: textfield;
}
</style>

{#snippet sharpInvertIcon()}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1.5em"
    height="1.5em"
    viewBox="0 0 24 24"
  >
    <!-- Icon from Google Material Icons by Material Design Authors - https://github.com/material-icons/material-icons/blob/master/LICENSE -->
    <path
      fill="currentColor"
      d="M12 4.81V19c-3.31 0-6-2.63-6-5.87c0-1.56.62-3.03 1.75-4.14zM6.35 7.56C4.9 8.99 4 10.96 4 13.13C4 17.48 7.58 21 12 21s8-3.52 8-7.87c0-2.17-.9-4.14-2.35-5.57L12 2z"
    />
  </svg>
{/snippet}

{#if status === "error"}
  <div class="status">PDF failed to load: {errorMsg}</div>
{:else}
  <div class="flex flex-col md:flex-row gap-8 mt-4">
    <!-- Sidebar -->
    <div class="bg-black p-4 border border-zinc-700 shadow-lg max-h-[24rem] md:max-h-[calc(100vh-24rem)] md:w-[300px] flex flex-col">
      <!-- Sticky header -->
      <div class="flex items-center gap-2 pb-4 mb-2 border-b border-zinc-800 sticky top-0 z-10 bg-black">
        <div class="w-1 h-1 bg-accent-500"></div>
        <h3 class="font-mono uppercase text-sm font-semibold text-accent-500">Outline</h3>
      </div>
      <!-- Scrollable list -->
      <div class="min-h-0 flex-1 overflow-y-auto">
        {#each flattenedOutline as x}
          <div class="flex items-center hover:bg-zinc-800 transition-all group">
            <button
              class="flex-1 text-left px-3 py-2 transition-all text-sm pl-[calc(var(--indent)*0.75rem)]"
              class:text-zinc-400={x.indent >= 1}
              style={`--indent:${x.indent + 1}`}
              onclick={() =>
              engine?.goToDest({
                doc: doc!,
                canvases: canvases!,
                dest: x.dest,
                scrollContainer: scrollContainer!,
                uiScale: 1,
              })}
              title={x.title}
            >
              {x.title}
            </button>
          </div>
        {/each}
      </div>
    </div>

    <!-- Main -->
    <div class="flex flex-col grow gap-4">
      <div class="flex gap-4">
        <div class="flex grow flex-row bg-black p-4 border border-zinc-700 shadow-lg gap-4">
          <div class="flex gap-2 items-center leading-none text-sm text-zinc-300 font-mono text-right">
            Page:
            <input
              type="number"
              min="1"
              max={pageCount}
              class="h-[1.25rem] bg-zinc-900 text-zinc-100 w-[5ch] text-right px-1 py-0.5 rounded border border-zinc-700 focus:outline-none focus:ring-1 focus:ring-accent-500"
              bind:value={currentPageInput}
              onchange={() => engine?.goToPage({ page: currentPageInput, pageCount, canvases })}
              onfocus={(e) => e.currentTarget.select()}
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  e.currentTarget.blur()
                  engine?.goToPage({ page: currentPageInput, pageCount, canvases })
                }
              }}
            />
            <div>/</div>
            <div>{pageCount}</div>
          </div>
          <div class="grow"></div>
          <button
            class="text-sm border-zinc-700 rounded hover:bg-zinc-800 transition"
            onclick={() => (inverted = !inverted)}
          >
            <span class:text-accent-500={inverted}>{@render sharpInvertIcon()}</span>
          </button>
        </div>
        <a
          class="hover:text-accent-400 pointer-events-auto bg-black flex items-center justify-center text-white border px-6 py-2 font-semibold font-mono text-md md:text-lg focus:ring-0 focus:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50"
          download={fileName}
          target="_blank"
          href={src}
        >
          Download
        </a>
      </div>

      <!-- Progress (no vertical space on ready) -->
      <div class="relative h-2 overflow-hidden">
        {#if status !== "ready"}
          {@const width = (pagesRendered / Math.max(1, pageCount)) * 100}
          <div
            class="relative h-2 rounded bg-zinc-800 w-full"
            out:fade={{ duration: 300 }}
          >
            <div
              class="absolute top-0 left-0 h-full bg-accent-500 transition-all rounded"
              style={`width: ${width}%`}
            >
            </div>
          </div>
        {/if}
      </div>

      <div
        class="h-[calc(100vh-8rem)] overflow-y-auto border-t border-b border-zinc-700 shadow-lg"
        bind:this={scrollContainer}
      >
        {#each Array(pageCount) as _, i (i)}
          <div class="page mb-8">
            <canvas
              class="block border border-zinc-700 shadow-lg"
              class:invert={inverted}
              bind:this={canvases[i]}
            ></canvas>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}
