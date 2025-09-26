<script lang="ts">
import EscherTextLogo from "$lib/components/icons/EscherTextLogo.svelte"
import Card from "$lib/components/ui/Card.svelte"

const CTA_URL = "https://www.escher.finance/"
const CTA_LABEL = "Visit Escher"
</script>

<Card class="relative overflow-hidden border-0 bg-white text-zinc-900 shadow-sm">
  <div class="absolute inset-0 bg-gradient-to-br from-[#f5f6ff] to-transparent" />
  <div class="absolute -right-16 top-1/2 hidden h-32 w-32 -translate-y-1/2 rotate-6 bg-[#0015ff]/10 blur-2xl md:block" />
  <div class="absolute -left-10 top-0 h-32 w-32 -rotate-6 bg-[#0015ff]/10 blur-3xl" />
  <div
    class="pointer-events-none absolute inset-0 opacity-30 mix-blend-multiply"
    style="background-image: url('data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 width=%22200%22 height=%22200%22 viewBox=%220 0 200 200%22%3E%3Cfilter id=%22n%22 x=%220%22 y=%220%22 width=%22100%25%22 height=%22100%25%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.9%22 numOctaves=%223%22 stitchTiles=%22stitch%22/%3E%3C/filter%3E%3Crect width=%22200%22 height=%22200%22 filter=%22url(%23n)%22 opacity=%220.55%22/%3E%3C/svg%3E'); background-size: 180px 180px;"
  />
  <div class="relative flex flex-col gap-4 px-6 py-5 md:flex-row md:items-center md:justify-between md:px-8 md:py-4">
    <div class="flex items-center gap-3">
      <span class="text-xs uppercase tracking-[0.32em] text-[#0015ff]">Powered by</span>
      <EscherTextLogo class="h-5 w-auto text-[#0015ff]" />
    </div>
    <a
      target="_blank"
      rel="noreferrer"
      href={CTA_URL}
      class="inline-flex items-center gap-2 rounded-md border border-[#0015ff]/30 bg-[#0015ff] px-4 py-2 text-xs font-semibold tracking-wide text-white transition hover:bg-[#000dd8]"
    >
      <span>{CTA_LABEL}</span>
      <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24">
        <path d="M7 17L17 7M7 7h10v10" />
      </svg>
    </a>
  </div>
</Card>
