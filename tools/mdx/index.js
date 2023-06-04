import fs from "node:fs/promises";
import { compile } from "@mdx-js/mdx";
import remarkGfm from "remark-gfm";

let args = process.argv;
let filename = args[3];
let input = await fs.readFile(filename);
let compiled = await compile(input, { remarkPlugins: [remarkGfm] });
console.log(String(compiled));
