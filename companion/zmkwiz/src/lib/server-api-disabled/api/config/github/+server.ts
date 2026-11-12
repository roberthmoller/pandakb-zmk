import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { parseGitHubUrl, rawGitHubUrl, repoTreeUrl } from '#lib/keymap/github';

const KEYMAP_CANDIDATES = ['lily58.keymap', 'corne.keymap', 'sofle.keymap'];

async function fetchText(url: string): Promise<{ ok: true; text: string } | { ok: false; status: number }> {
	const res = await fetch(url, {
		headers: {
			Accept: 'application/vnd.github.raw',
			'User-Agent': 'ZmkWiz'
		}
	});
	if (!res.ok) return { ok: false, status: res.status };
	return { ok: true, text: await res.text() };
}

export const GET: RequestHandler = async ({ url }) => {
	const input = url.searchParams.get('url') ?? url.searchParams.get('repo');
	if (!input) throw error(400, 'url query required');

	let ref;
	try {
		ref = parseGitHubUrl(input);
	} catch (e) {
		throw error(400, e instanceof Error ? e.message : String(e));
	}

	let keymapText: string | null = null;
	let keymapFile: string | null = null;

	for (const candidate of KEYMAP_CANDIDATES) {
		const result = await fetchText(rawGitHubUrl(ref, candidate));
		if (result.ok) {
			keymapText = result.text;
			keymapFile = candidate;
			break;
		}
	}

	if (!keymapText || !keymapFile) {
		// Last resort: list via contents API is heavier; try generic *.keymap name from query.
		const named = url.searchParams.get('file');
		if (named) {
			const result = await fetchText(rawGitHubUrl(ref, named));
			if (result.ok) {
				keymapText = result.text;
				keymapFile = named;
			}
		}
	}

	if (!keymapText || !keymapFile) {
		throw error(
			404,
			`No .keymap found under ${ref.configDir}/ (tried ${KEYMAP_CANDIDATES.join(', ')}).`
		);
	}

	let layoutJson: unknown = null;
	const layoutName = keymapFile.replace(/\.keymap$/, '.json');
	const layoutResult = await fetchText(rawGitHubUrl(ref, layoutName));
	if (layoutResult.ok) {
		try {
			layoutJson = JSON.parse(layoutResult.text);
		} catch {
			layoutJson = null;
		}
	}

	return json({
		ref,
		source: repoTreeUrl(ref),
		keymapFile,
		keymap: keymapText,
		layout: layoutJson
	});
};
