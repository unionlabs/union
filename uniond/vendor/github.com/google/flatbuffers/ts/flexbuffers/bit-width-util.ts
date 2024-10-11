import { BitWidth } from './bit-width.js'

export function toByteWidth(bitWidth: BitWidth): number {
  return 1 << bitWidth;
}

export function iwidth(value: number | bigint): BitWidth {
  if (value >= -128 && value <= 127) return BitWidth.WIDTH8;
  if (value >= -32768 && value <= 32767) return BitWidth.WIDTH16;
  if (value >= -2147483648 && value <= 2147483647) return BitWidth.WIDTH32;
  return BitWidth.WIDTH64;
}

export function fwidth(value: number): BitWidth {
  return value === Math.fround(value) ? BitWidth.WIDTH32 : BitWidth.WIDTH64;
}

export function uwidth(value: number): BitWidth {
  if (value <= 255) return BitWidth.WIDTH8;
  if (value <= 65535) return BitWidth.WIDTH16;
  if (value <= 4294967295) return BitWidth.WIDTH32;
  return BitWidth.WIDTH64;
}

export function fromByteWidth(value: number): BitWidth {
  if (value === 1) return BitWidth.WIDTH8;
  if (value === 2) return BitWidth.WIDTH16;
  if (value === 4) return BitWidth.WIDTH32;
  return BitWidth.WIDTH64;
}

export function paddingSize(bufSize: number, scalarSize: number): number {
  return (~bufSize + 1) & (scalarSize - 1);
}