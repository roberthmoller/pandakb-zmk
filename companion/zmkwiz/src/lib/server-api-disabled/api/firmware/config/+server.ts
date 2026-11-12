import { json, error } from '@sveltejs/kit';
import { access } from 'node:fs/promises';
import { join } from 'node:path';
import type { RequestHandler } from './$types';
import { assertSafeWestRoot } from '#lib/firmware/westPath';

export const GET: RequestHandler = async ({ url }) => {
	const rootParam = url.searchParams.get('root');
	if (!rootParam) {
		throw error(400, 'root query required');
	}
	let root: string;
	try {
		root = assertSafeWestRoot(rootParam);
	} catch (e) {
		throw error(400, e instanceof Error ? e.message : String(e));
	}

	const keymap = join(root, 'config/lily58.keymap');
	const leftUf2 = join(root, 'build/lily58_left/zephyr/zmk.uf2');
	const rightUf2 = join(root, 'build/lily58_right/zephyr/zmk.uf2');

	const exists = async (p: string) => {
		try {
			await access(p);
			return true;
		} catch {
			return false;
		}
	};

	return json({
		root,
		keymap: (await exists(keymap)) ? keymap : null,
		artifacts: {
			left: (await exists(leftUf2)) ? leftUf2 : null,
			right: (await exists(rightUf2)) ? rightUf2 : null
		},
		bootloaderMounted: await exists('/Volumes/RPI-RP2')
	});
};
