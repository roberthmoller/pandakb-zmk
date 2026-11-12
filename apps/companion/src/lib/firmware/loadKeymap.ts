import { parseKeymap } from '#lib/keymap/parseKeymap';
import { boardState, type ConfigOrigin } from '#lib/state/board.svelte';
import { getFirmwareKeymap, isTauri, setFirmwareKeymap, type KeymapFile } from './tauri';

export type LoadFirmwareKeymapResult = {
	ok: boolean;
	path?: string;
	layerCount?: number;
	warnings?: string[];
	error?: string;
};

function applyKeymapText(contents: string, origin: ConfigOrigin): LoadFirmwareKeymapResult {
	const keyCount = boardState.board.keys.length;
	const parsed = parseKeymap(contents, { keyCount });
	boardState.applyLayers(parsed.layers, origin, parsed.warnings);
	return {
		ok: true,
		path: origin.kind === 'firmware' ? origin.path : undefined,
		layerCount: parsed.layers.length,
		warnings: parsed.warnings
	};
}

/**
 * Load layers from the repo's `config/Lily58.keymap` (companion source of truth).
 * Falls back to the bundled stock keymap when not running in Tauri / path missing.
 */
export async function loadFirmwareKeymap(): Promise<LoadFirmwareKeymapResult> {
	if (!isTauri()) {
		return { ok: false, error: 'Not running in Tauri' };
	}
	try {
		const file = await getFirmwareKeymap();
		const origin: ConfigOrigin = {
			kind: 'firmware',
			label: 'Local firmware',
			path: file.path
		};
		const result = applyKeymapText(file.contents, origin);
		return { ...result, path: file.path };
	} catch (e) {
		return { ok: false, error: e instanceof Error ? e.message : String(e) };
	}
}

/** Write raw keymap text to the firmware path and re-parse into board state. */
export async function saveFirmwareKeymap(contents: string): Promise<LoadFirmwareKeymapResult> {
	if (!isTauri()) {
		return { ok: false, error: 'Not running in Tauri' };
	}
	try {
		const file: KeymapFile = await setFirmwareKeymap(contents);
		const origin: ConfigOrigin = {
			kind: 'firmware',
			label: 'Local firmware',
			path: file.path
		};
		const result = applyKeymapText(file.contents, origin);
		return { ...result, path: file.path };
	} catch (e) {
		return { ok: false, error: e instanceof Error ? e.message : String(e) };
	}
}
