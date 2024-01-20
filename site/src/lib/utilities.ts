export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export const arraySizeN = (n: number) => Array.from(Array(n).keys());

export const roundNumber = (_number: number, decimalPlaces: number) =>
  Math.round(_number * 10 ** decimalPlaces) / 10 ** decimalPlaces;
