export function is_array_values_supported() {
  return typeof Array.prototype.values === 'function';
}
