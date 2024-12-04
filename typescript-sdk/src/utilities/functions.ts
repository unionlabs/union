/**
 * Creates a new composite function that invokes the functions from right to left
 */

export default function compose<T>(...functions: Array<(...args: Array<T>) => T>) {
  return functions.reduce(
    (acc, currentFn) =>
      (...args: Array<T>) =>
        acc(currentFn(...args))
  )
}
