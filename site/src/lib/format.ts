export const toFixedPoint = (value: bigint, decimals: number) => {
	let right = BigInt(value) % BigInt(10 ** decimals);
	let left = BigInt(value) - right;

	return left.toString().concat('.', right.toString().padStart(decimals, '0'));
};
