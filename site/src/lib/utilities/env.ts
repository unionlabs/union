export function getEnvVar(name: string, defaultValue: string = ""): string {
  if (typeof process !== "undefined" && process.env[name]) {
    return process.env[name]!;
  }
  if (typeof window !== "undefined" && (window as any).__env?.[name]) {
    return (window as any).__env[name];
  }
  return defaultValue;
}
