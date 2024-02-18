import { visit } from "unist-util-visit";
import type { RemarkPlugin } from "@astrojs/markdown-remark";

/**
 * This is a remark plugin. It's being used in markdown.config.ts
 */

const escapeMap: Record<string, string> = {
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#39;",
};

const escapeHtml = (str: string) =>
  str.replace(/[&<>"']/g, (c) => escapeMap[c]);

export const mermaid: RemarkPlugin<[]> = () => (tree) => {
  visit(tree, "code", (node) => {
    if (node.lang !== "mermaid") return;
    // @ts-expect-error
    node.type = "html";
    node.value = `<div class="mermaid">${escapeHtml(node.value)}</div>`;
  });
};
