// Genera deploy-hosting/latest.json: el manifiesto que lee el actualizador
// (se sube a la release de GitHub junto al setup).
import { readFileSync, writeFileSync } from 'node:fs';

const { version } = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const setup = `Asegurao_${version}_x64-setup.exe`;
const signature = readFileSync(`src-tauri/target/release/bundle/nsis/${setup}.sig`, 'utf8').trim();

const manifest = {
  version,
  notes: `Asegurao ${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': {
      signature,
      url: `https://github.com/PoxiiTV/Asegurao/releases/download/v${version}/${setup}`
    }
  }
};
writeFileSync('deploy-hosting/latest.json', JSON.stringify(manifest, null, 2));
console.log(`latest.json -> v${version}`);
