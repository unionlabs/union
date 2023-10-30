const fs = require('fs');
const url = require('url');

exports.read_file = (str) => fs.readFileSync(url.fileURLToPath(str), "utf8");
