import url from "node:url";
import path from "node:path";
import fs from "node:fs/promises";

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export const arraySizeN = (n: number) => Array.from(Array(n).keys());

export const roundNumber = (_number: number, decimalPlaces: number) =>
  Math.round(_number * 10 ** decimalPlaces) / 10 ** decimalPlaces;

export const generateRandomNumber = (min: number, max: number) =>
  Math.random() * (max - min) + min;

export async function getFileContent({ filepath }: { filepath: string }) {
  const file = path.resolve(__dirname, filepath);
  if (!fs.stat(file)) throw new Error(`File not found: ${file}`);
  return fs.readFile(file, { encoding: "utf8" });
}
