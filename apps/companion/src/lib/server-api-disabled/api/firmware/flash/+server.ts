import { json, error } from '@sveltejs/kit';
import { access, copyFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { RequestHandler } from './$types';
import { assertSafeWestRoot } from '#lib/firmware/westPath';

const BOOT = '/Volumes/RPI-RP2';

export const GET: RequestHandler = async () => {
	try {
		await access(BOOT);
		return json({ mounted: true, path: BOOT });
	} catch {
		return json({ mounted: false, path: BOOT });
	}
};

/**
 * Body: { root, half: 'left' | 'right' }
 * Copies the half's UF2 onto the mounted nice!nano bootloader volume.
 */
export const POST: RequestHandler = async ({ request }) => {
	const body = (await request.json().catch(() => ({}))) as {
		root?: string;
		half?: 'left' | 'right';
	};
	if (!body.root || (body.half !== 'left' && body.half !== 'right')) {
		throw error(400, 'root and half (left|right) required');
	}

	let root: string;
	try {
		root = assertSafeWestRoot(body.root);
	} catch (e) {
		throw error(400, e instanceof Error ? e.message : String(e));
	}

	const uf2 = join(
		root,
		body.half === 'left' ? 'build/lily58_left/zephyr/zmk.uf2' : 'build/lily58_right/zephyr/zmk.uf2'
	);

	try {
		await access(uf2);
	} catch {
		throw error(404, `UF2 missing: ${uf2} — build first`);
	}

	try {
		await access(BOOT);
	} catch {
		throw error(
			409,
			'Bootloader not mounted at /Volumes/RPI-RP2 — double-reset the half, then retry'
		);
	}

	const dest = join(BOOT, `zmk_${body.half}.uf2`);
	await copyFile(uf2, dest);
	return json({ ok: true, from: uf2, to: dest, half: body.half });
};
