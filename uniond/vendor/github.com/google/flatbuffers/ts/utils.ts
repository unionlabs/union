export const int32 = new Int32Array(2);
export const float32 = new Float32Array(int32.buffer);
export const float64 = new Float64Array(int32.buffer);
export const isLittleEndian = new Uint16Array(new Uint8Array([1, 0]).buffer)[0] === 1;