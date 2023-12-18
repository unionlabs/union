export const toFixedPoint = (value: bigint, decimals: number) => {
  let right = BigInt(value) % BigInt(10) ** BigInt(decimals);
  let left = BigInt(value) / BigInt(10) ** BigInt(decimals);

  return left.toString().concat(".", right.toString().padStart(decimals, "0"));
};

export const toFixedEth = (value: bigint) => {
  return toFixedPoint(value, 18).slice(0, -12);
};

export const toFixedUno = (value: bigint | null) => {
  if (!value) return
  return toFixedPoint(value, 6);
};
