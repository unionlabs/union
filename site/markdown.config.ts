import remarkToc from "remark-toc";
import rehypeSlug from "rehype-slug";
import remarkMathPlugin from "remark-math";
import rehypeKatexPlugin from "rehype-katex";
import rehypeMathjaxPlugin from "rehype-mathjax";
import remarkSmartypants from "remark-smartypants";
import { type AstroUserConfig } from "astro/config";
import { rehypeHeadingIds } from "@astrojs/markdown-remark";
import rehypeAutolinkHeadings from "rehype-autolink-headings";
import remarkMermaid from "astro-diagram/remark-mermaid";

type Markdown = AstroUserConfig["markdown"];

export const markdownConfiguration = {
  gfm: true,
  smartypants: false,
  remarkPlugins: [
    remarkMathPlugin,
    remarkSmartypants,
    remarkMermaid,
    [remarkToc, { heading: "contents", prefix: "toc-" }],
  ],
  rehypePlugins: [
    rehypeHeadingIds,
    rehypeSlug,
    [rehypeAutolinkHeadings, { behavior: "wrap" }],
    rehypeKatexPlugin,
    rehypeMathjaxPlugin,
  ],
} satisfies Markdown;
