export const toFixedPoint = (value: bigint, decimals: number) => {
	console.log("value: " + value)
	let right = BigInt(value) % BigInt(10) ** BigInt(decimals);
	console.log("right: " + right)
	let left = BigInt(value) / BigInt(10) ** BigInt(decimals);
	console.log("left: " + left)

	return left.toString().concat('.', right.toString().padStart(decimals, '0'));
};

export const toFixedEth = (value: bigint) => {
	return toFixedPoint(value, 18).slice(0, -12)
}

export const toFixedUno = (value: bigint) => {
	return toFixedPoint(value, 6)
}
