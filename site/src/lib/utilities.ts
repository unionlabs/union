export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export const arraySizeN = (n: number) => Array.from(Array(n).keys());
