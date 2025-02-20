import fs from 'fs';

// ts-node convert.ts
// Function to convert JSON file to TypeScript variable after running hubble command
function convertJsonToTs(jsonPath: string, variableName: string): string {
  const jsonContent = fs.readFileSync(jsonPath, 'utf-8');
  const obj = JSON.parse(jsonContent);
  return JSON.stringify(obj, null, 2)
    .replace(/"([^"]+)":/g, '$1:')
    .replace(/^/, `export const ${variableName} = `)
    .replace(/}$/, '};');
}

const files = [
  { path: './app.ucs01.json', name: 'ucs01Abi' },
  { path: './app.ucs02.json', name: 'ucs02Abi' },
  { path: './app.ucs03.json', name: 'ucs03Abi' },
];

files.forEach(file => {
  const tsContent = convertJsonToTs(file.path, file.name);
  fs.writeFileSync(`${file.name}.ts`, tsContent);
});