import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseKeymap } from '../src/lib/keymap/parseKeymap';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const repoKeymap = join(root, '../../config/Lily58.keymap');
const outPath = join(root, 'src/lib/boards/lily58/keymap.json');

const source = readFileSync(repoKeymap, 'utf8');
const { layers, warnings } = parseKeymap(source, { keyCount: 58 });
console.log(
	'layers:',
	layers.map((l, i) => `${i}:${l.id}/${l.name}(${l.bindings.length})`).join(', ')
);
if (warnings.length) console.log('warnings:', warnings);

writeFileSync(
	outPath,
	JSON.stringify(
		{
			source: 'config/Lily58.keymap',
			notes:
				'Generated from repo config/Lily58.keymap — companion prefers the live file via firmware_keymap_get.',
			layers
		},
		null,
		2
	) + '\n'
);
console.log('wrote', outPath);
