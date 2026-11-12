import { bindingFromRaw } from '#lib/keymap/labels';
import type { Binding, Layer } from '#lib/keymap/types';

/** ZMK keycode aliases → canonical names used by HID / event maps. */
const CODE_ALIASES: Record<string, string> = {
	LEFT_ARROW: 'LEFT',
	RIGHT_ARROW: 'RIGHT',
	UP_ARROW: 'UP',
	DOWN_ARROW: 'DOWN',
	K_MUTE: 'C_MUTE'
};

const LAYER_ID_HINTS: { match: RegExp; id: string; name: string }[] = [
	{ match: /^(default|base|layer_?0)$/i, id: 'base', name: 'Base' },
	{ match: /^(lower|layer_?1)$/i, id: 'lower', name: 'Lower' },
	{ match: /^(upper|raise|layer_?2)$/i, id: 'raise', name: 'Raise' },
	{ match: /^(adjust|layer_?3)$/i, id: 'adjust', name: 'Adjust' }
];

export type ParsedKeymap = {
	layers: Layer[];
	warnings: string[];
};

export type ParseKeymapOptions = {
	/** Expected binding count per layer (Lily58 = 58). */
	keyCount?: number;
};

function stripComments(source: string): string {
	return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
}

function normalizeCode(code: string): string {
	return CODE_ALIASES[code] ?? code;
}

function normalizeBindingRaw(raw: string): string {
	const trimmed = raw.trim();
	if (!trimmed.startsWith('&')) return trimmed;
	const parts = trimmed.replace(/^&/, '').split(/\s+/);
	const behavior = parts[0] ?? '';
	const params = parts.slice(1).map(normalizeCode);
	if (params.length === 0) return `&${behavior}`;
	return `&${behavior} ${params.join(' ')}`;
}

function tokenizeBindings(inner: string): string[] {
	const collapsed = inner.replace(/\s+/g, ' ').trim();
	if (!collapsed) return [];
	return collapsed
		.split(/(?=&)/)
		.map((part) => part.trim())
		.filter((part) => part.startsWith('&'));
}

function layerMetaFromNode(nodeName: string, body: string): { id: string; name: string } {
	const display = /display-name\s*=\s*"([^"]+)"/.exec(body)?.[1];
	const stem = nodeName.replace(/_layer$/i, '');
	for (const hint of LAYER_ID_HINTS) {
		if (hint.match.test(stem) || hint.match.test(nodeName)) {
			return { id: hint.id, name: display ?? hint.name };
		}
	}
	const id = stem.toLowerCase().replace(/[^a-z0-9]+/g, '-') || 'layer';
	const name = display ?? stem.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
	return { id, name };
}

/** Match brace-balanced `{ ... }` starting at `openIndex` (must point at `{`). */
function sliceBalancedBlock(source: string, openIndex: number): string | null {
	if (source[openIndex] !== '{') return null;
	let depth = 0;
	for (let i = openIndex; i < source.length; i++) {
		const ch = source[i];
		if (ch === '{') depth += 1;
		else if (ch === '}') {
			depth -= 1;
			if (depth === 0) return source.slice(openIndex + 1, i);
		}
	}
	return null;
}

/**
 * Prefer the `compatible = "zmk,keymap"` node body; fall back to the whole file.
 */
function extractKeymapBody(stripped: string): { body: string; warnings: string[] } {
	const warnings: string[] = [];
	const compat = /compatible\s*=\s*"zmk,keymap"\s*;/.exec(stripped);
	if (!compat) {
		warnings.push('No compatible = "zmk,keymap" found; scanning the whole file.');
		return { body: stripped, warnings };
	}

	// Walk back to the keymap node's opening brace… or forward from the property.
	const after = stripped.indexOf('{', compat.index);
	// The `{` for the keymap node is usually *before* compatible. Find nearest `{` before compat
	// that opens the node containing this property.
	const before = stripped.lastIndexOf('{', compat.index);
	const open = before >= 0 ? before : after;
	if (open < 0) {
		warnings.push('Could not locate keymap node braces; scanning the whole file.');
		return { body: stripped, warnings };
	}

	const body = sliceBalancedBlock(stripped, open);
	if (!body) {
		warnings.push('Unbalanced braces in keymap node; scanning the whole file.');
		return { body: stripped, warnings };
	}
	return { body, warnings };
}

function findLayerNodes(keymapBody: string): { name: string; body: string }[] {
	const nodes: { name: string; body: string }[] = [];
	const re = /(\w+)\s*\{/g;
	let match: RegExpExecArray | null;
	while ((match = re.exec(keymapBody)) !== null) {
		const name = match[1]!;
		const open = match.index + match[0].length - 1;
		const body = sliceBalancedBlock(keymapBody, open);
		if (body == null) continue;

		// Skip nested property-only blocks we don't care about by requiring bindings=
		if (!/(?<![\w-])bindings\s*=/.test(body)) continue;
		if (/status\s*=\s*"reserved"/.test(body)) continue;

		nodes.push({ name, body });
		// Don't re-scan inside this node
		re.lastIndex = open + body.length + 2;
	}
	return nodes;
}

function extractBindingsList(body: string): string | null {
	// Only the key bindings property — not sensor-bindings.
	const match = /(?<![\w-])bindings\s*=\s*<([^>]*)>/.exec(body);
	return match?.[1] ?? null;
}

/**
 * Parse a ZMK `.keymap` Devicetree overlay into labeled layers.
 * Handles the common Keymap Editor / PandaKB shape; does not resolve `#include`s.
 */
export function parseKeymap(source: string, options: ParseKeymapOptions = {}): ParsedKeymap {
	const keyCount = options.keyCount;
	const stripped = stripComments(source);
	const { body: keymapBody, warnings } = extractKeymapBody(stripped);
	const layers: Layer[] = [];
	const usedIds = new Set<string>();

	for (const node of findLayerNodes(keymapBody)) {
		const inner = extractBindingsList(node.body);
		if (inner == null) continue;

		const raws = tokenizeBindings(inner).map(normalizeBindingRaw);
		if (raws.length === 0) {
			warnings.push(`Layer node "${node.name}" has an empty bindings list.`);
			continue;
		}

		// Combos and one-off nodes often have a handful of bindings — skip unless keyCount matches
		// or the count is clearly a full layer (≥ half the board when keyCount known).
		if (keyCount != null) {
			if (raws.length !== keyCount) {
				if (raws.length < Math.max(8, Math.floor(keyCount / 2))) {
					continue;
				}
				warnings.push(
					`Layer "${node.name}" has ${raws.length} bindings; expected ${keyCount} for this board.`
				);
			}
		} else if (raws.length < 8) {
			continue;
		}

		const meta = layerMetaFromNode(node.name, node.body);
		let id = meta.id;
		if (usedIds.has(id)) {
			let n = 2;
			while (usedIds.has(`${id}-${n}`)) n += 1;
			id = `${id}-${n}`;
			warnings.push(`Duplicate layer id "${meta.id}" renamed to "${id}".`);
		}
		usedIds.add(id);

		const bindings: Binding[] = raws.map((raw) => bindingFromRaw(raw));
		layers.push({ id, name: meta.name, bindings });
	}

	if (layers.length === 0) {
		throw new Error(
			'No keymap layers found. Expected a zmk,keymap node with bindings = < ... >; blocks.'
		);
	}

	return { layers, warnings };
}
