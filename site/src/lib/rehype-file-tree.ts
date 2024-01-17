import { rehype } from "rehype";
import type { Element } from "hast";
import { h, type Child } from "hastscript";
import { fromHtml } from "hast-util-from-html";
import { toString } from "hast-util-to-string";
import { getIcon } from "./file-tree-icons.ts";
import { CONTINUE, SKIP, visit } from "unist-util-visit";

/** Make a text node with the pass string as its contents. */
const Text = (value = ""): { type: "text"; value: string } => ({
  type: "text",
  value,
});

/** Convert an HTML string containing an SVG into a HAST element node. */
const makeSVGIcon = (svgString: string) => {
  const root = fromHtml(svgString, { fragment: true });
  const svg = root.children[0] as Element;
  svg.properties = {
    ...svg.properties,
    width: 16,
    height: 16,
    class: "tree-icon",
    "aria-hidden": "true",
  };
  return svg;
};

const FileIcon = (filename: string) => {
  const { svg } = getIcon(filename);
  return makeSVGIcon(svg);
};

const FolderIcon = makeSVGIcon(
  '<svg viewBox="0 0 20 20"><path d="M14.77 6.45H9.8v-.47A.97.97 0 0 0 8.83 5H3.75v10H15.7V7.42a.91.91 0 0 0-.93-.97Z"/></svg>'
);

export const fileTreeProcessor = rehype().use(() => (tree: Element, file) => {
  const { directoryLabel } = file.data as { directoryLabel: string };
  visit(tree, "element", (node) => {
    if (node.type !== "element") return CONTINUE;
    // Strip nodes that only contain newlines
    node.children = node.children.filter(
      (child) =>
        child.type === "comment" ||
        child.type !== "text" ||
        !/^\n+$/.test(child.value)
    );

    if (node.tagName !== "li") return CONTINUE;

    // Ensure node has properties so we can assign classes later.
    if (!node.properties) node.properties = {};

    const [firstChild, ...otherChildren] = node.children;

    const comment: Child[] = [];
    if (firstChild?.type === "text") {
      const [filename, ...fragments] = firstChild.value.split(" ");
      firstChild.value = filename || "";
      comment.push(fragments.join(" "));
    }
    const subTreeIndex = otherChildren.findIndex(
      (child) => child.type === "element" && child.tagName === "ul"
    );
    const commentNodes =
      subTreeIndex > -1
        ? otherChildren.slice(0, subTreeIndex)
        : [...otherChildren];
    otherChildren.splice(
      0,
      subTreeIndex > -1 ? subTreeIndex : otherChildren.length
    );
    comment.push(...commentNodes);

    const firstChildTextContent = firstChild ? toString(firstChild) : "";

    // Decide a node is a directory if it ends in a `/` or contains another list.
    const isDirectory =
      /\/\s*$/.test(firstChildTextContent) ||
      otherChildren.some(
        (child) => child.type === "element" && child.tagName === "ul"
      );
    const isPlaceholder = /^\s*(\.{3}|…)\s*$/.test(firstChildTextContent);
    const isHighlighted =
      firstChild?.type === "element" && firstChild.tagName === "strong";
    const hasContents = otherChildren.length > 0;

    const fileExtension = isDirectory
      ? "dir"
      : firstChildTextContent.trim().split(".").pop() || "";

    const icon = h(
      "span",
      isDirectory ? FolderIcon : FileIcon(firstChildTextContent)
    );
    if (!icon.properties) icon.properties = {};
    if (isDirectory) {
      icon.children.unshift(h("span", { class: "sr-only" }, directoryLabel));
    }

    node.properties.class = isDirectory ? "directory" : "file";
    if (isPlaceholder) node.properties.class += " empty";
    node.properties["data-filetype"] = fileExtension;

    const treeEntry = h(
      "span",
      { class: "tree-entry" },
      h("span", { class: isHighlighted ? "highlight" : "" }, [
        isPlaceholder ? null : icon,
        firstChild,
      ]),
      Text(comment.length > 0 ? " " : ""),
      comment.length > 0 ? h("span", { class: "comment" }, ...comment) : Text()
    );

    if (isDirectory) {
      node.children = [
        h("details", { open: hasContents }, [
          h("summary", treeEntry),
          ...(hasContents ? otherChildren : [h("ul", h("li", "…"))]),
        ]),
      ];
      // Continue down the tree.
      return CONTINUE;
    }

    node.children = [treeEntry, ...otherChildren];

    // Files can’t contain further files or directories, so skip iterating children.
    return SKIP;
  });
});
