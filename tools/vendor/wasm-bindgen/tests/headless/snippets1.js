export function get_two() {
  return 2;
}

let a = 0;
export function get_stateful() {
  a += 1;
  return a;
}
