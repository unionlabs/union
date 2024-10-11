const readline = require('readline');

// Create readline interface
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

let content = '';

// Read stdin content
rl.on('line', (line) => {
  content += line + '\n';
});

rl.on('close', () => {
  // Escape special characters
  const escapedContent = content.replace(/"/g, ' ');
  // Convert the JSON object to a string
  const jsonStr = JSON.stringify(escapedContent);
  const jsonStr2 = jsonStr.slice(1, -1);
  console.log(jsonStr2);
});