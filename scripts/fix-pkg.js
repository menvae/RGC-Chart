const fs = require('fs');
const path = require('path');

const Keywords = [
  "wasm",
  "rust",
  "rhythm-game",
  "osu",
  "stepmania",
  "parser",
  "converter",
  "writer"
];

const nodePackagePath = path.join(__dirname, '../dist-node/package.json');
const nodePackage = JSON.parse(fs.readFileSync(nodePackagePath, 'utf8'));
nodePackage.name = 'rgc-chart-nodejs';
nodePackage.keywords = Keywords;
fs.writeFileSync(nodePackagePath, JSON.stringify(nodePackage, null, 2));
console.log('Node package name and keywords updated');

const webPackagePath = path.join(__dirname, '../dist-web/package.json');
const webPackage = JSON.parse(fs.readFileSync(webPackagePath, 'utf8'));
webPackage.name = 'rgc-chart-browser';
webPackage.keywords = Keywords;
fs.writeFileSync(webPackagePath, JSON.stringify(webPackage, null, 2));
console.log('Web package name and keywords updated');

console.log('All package names and keywords updated');