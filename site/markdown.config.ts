import remarkToc from "remark-toc";
import { visit } from "unist-util-visit";
import { toHtml } from "hast-util-to-html";
import remarkMathPlugin from "remark-math";
import { toHast } from "mdast-util-to-hast";
import rehypeKatexPlugin from "rehype-katex";
import { type AstroUserConfig } from "astro/config";
import { rehypeHeadingIds } from "@astrojs/markdown-remark";

type Markdown = AstroUserConfig["markdown"];

export const markdownConfiguration = {
  gfm: true,
  remarkPlugins: [remarkMathPlugin, [remarkToc, { heading: "contents" }]],
  rehypePlugins: [rehypeHeadingIds, rehypeKatexPlugin],
} satisfies Markdown;

function remarkMermaidToHtml() {
  // @ts-ignore
  return (tree, _file) => {
    visit(tree, "code", (code, index, parent) => {
      if (index === null || parent === null) return;
      if (code.lang === "mermaid") {
        const hast = toHast(code);
        const html = toHtml(hast);
        parent.children.splice(parent.children.indexOf(code), 1, {
          type: "html",
          value: html,
        });
      }
    });
  };
}
