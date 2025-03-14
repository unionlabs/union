export function greet(name: string): string {
  return `Hello, ${name}!`
}

// Export EVM module
export * as evm from './evm';
