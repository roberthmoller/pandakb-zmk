/** Map KeyboardEvent.code → ZMK keycode used in Lily58 bindings. */
export const EVENT_CODE_TO_ZMK: Record<string, string> = {
	KeyA: 'A',
	KeyB: 'B',
	KeyC: 'C',
	KeyD: 'D',
	KeyE: 'E',
	KeyF: 'F',
	KeyG: 'G',
	KeyH: 'H',
	KeyI: 'I',
	KeyJ: 'J',
	KeyK: 'K',
	KeyL: 'L',
	KeyM: 'M',
	KeyN: 'N',
	KeyO: 'O',
	KeyP: 'P',
	KeyQ: 'Q',
	KeyR: 'R',
	KeyS: 'S',
	KeyT: 'T',
	KeyU: 'U',
	KeyV: 'V',
	KeyW: 'W',
	KeyX: 'X',
	KeyY: 'Y',
	KeyZ: 'Z',
	Digit0: 'N0',
	Digit1: 'N1',
	Digit2: 'N2',
	Digit3: 'N3',
	Digit4: 'N4',
	Digit5: 'N5',
	Digit6: 'N6',
	Digit7: 'N7',
	Digit8: 'N8',
	Digit9: 'N9',
	Space: 'SPACE',
	Enter: 'RET',
	Backspace: 'BSPC',
	Tab: 'TAB',
	Escape: 'ESC',
	Minus: 'MINUS',
	Equal: 'EQUAL',
	BracketLeft: 'LBKT',
	BracketRight: 'RBKT',
	Backslash: 'BSLH',
	Semicolon: 'SEMI',
	Quote: 'SQT',
	Comma: 'COMMA',
	Period: 'DOT',
	Slash: 'FSLH',
	Backquote: 'GRAVE',
	ShiftLeft: 'LSHFT',
	ShiftRight: 'RSHFT',
	ControlLeft: 'LCTRL',
	ControlRight: 'RCTRL',
	AltLeft: 'LALT',
	AltRight: 'RALT',
	MetaLeft: 'LGUI',
	MetaRight: 'RGUI',
	ArrowLeft: 'LEFT',
	ArrowRight: 'RIGHT',
	ArrowUp: 'UP',
	ArrowDown: 'DOWN',
	F1: 'F1',
	F2: 'F2',
	F3: 'F3',
	F4: 'F4',
	F5: 'F5',
	F6: 'F6',
	F7: 'F7',
	F8: 'F8',
	F9: 'F9',
	F10: 'F10',
	F11: 'F11',
	F12: 'F12'
};

/** Characters that ZMK often emits as dedicated symbol keycodes (Lower layer). */
export const EVENT_KEY_TO_ZMK: Record<string, string> = {
	'!': 'EXCL',
	'@': 'AT',
	'#': 'HASH',
	$: 'DOLLAR',
	'%': 'PRCNT',
	'^': 'CARET',
	'&': 'AMPS',
	'*': 'ASTRK',
	'(': 'LPAR',
	')': 'RPAR',
	'~': 'TILDE',
	'{': 'LBRC',
	'}': 'RBRC',
	'|': 'PIPE',
	'+': 'PLUS'
};

export type BindingHit = {
	layerId: string;
	index: number;
	code: string;
};

export type ResolvedPress = {
	index: number;
	layerId: string;
	code: string;
	/** True when this HID is unique to a non-base layer (or strongly implies one). */
	impliesLayer: boolean;
	shiftHeld: boolean;
};

export function buildCodeIndexMap(
	bindings: { behavior: string; params: string[] }[]
): Map<string, number> {
	const zmkToIndex = new Map<string, number>();
	for (let index = 0; index < bindings.length; index++) {
		const binding = bindings[index];
		if (binding.behavior !== 'kp') continue;
		const code = binding.params[0];
		if (code && !zmkToIndex.has(code)) zmkToIndex.set(code, index);
	}

	const map = new Map<string, number>();
	for (const [eventCode, zmk] of Object.entries(EVENT_CODE_TO_ZMK)) {
		const index = zmkToIndex.get(zmk);
		if (index !== undefined) map.set(eventCode, index);
	}
	return map;
}

/** Indices of momentary layer keys on the base layer. */
export function findMomentaryKeys(
	bindings: { behavior: string; params: string[] }[]
): { index: number; layer: number }[] {
	const keys: { index: number; layer: number }[] = [];
	for (let index = 0; index < bindings.length; index++) {
		const binding = bindings[index];
		if (
			(binding.behavior === 'mo' || binding.behavior === 'mo_prev') &&
			binding.params[0] != null
		) {
			keys.push({ index, layer: Number(binding.params[0]) });
		}
	}
	return keys;
}

export function buildZmkHitMap(
	layers: { id: string; bindings: { behavior: string; params: string[] }[] }[]
): Map<string, BindingHit[]> {
	const map = new Map<string, BindingHit[]>();
	for (const layer of layers) {
		layer.bindings.forEach((binding, index) => {
			if (binding.behavior !== 'kp') return;
			const code = binding.params[0];
			if (!code) return;
			const hits = map.get(code) ?? [];
			hits.push({ layerId: layer.id, index, code });
			map.set(code, hits);
		});
	}
	return map;
}

function pickHit(
	hits: BindingHit[],
	preferredLayerId: string | null,
	activeLayerId: string
): BindingHit | null {
	if (hits.length === 0) return null;
	if (preferredLayerId) {
		const preferred = hits.find((hit) => hit.layerId === preferredLayerId);
		if (preferred) return preferred;
	}
	const active = hits.find((hit) => hit.layerId === activeLayerId);
	if (active) return active;
	const nonBase = hits.filter((hit) => hit.layerId !== 'base');
	if (nonBase.length === 1) return nonBase[0] ?? null;
	if (nonBase.length > 1) {
		const lower = nonBase.find((hit) => hit.layerId === 'lower');
		if (lower) return lower;
		return nonBase[0] ?? null;
	}
	return hits[0] ?? null;
}

/**
 * Resolve a keyboard event to a board key + layer.
 * Real Lily58 `&mo` keys never reach the host — layer is inferred from HID
 * (arrows → Raise, F-keys / Lower symbols → Lower) or from AltLeft/AltRight hold.
 *
 * ZMK `&kp EXCL` sets the Shift modifier bit without a Shift keydown. A real
 * Shift+1 presses ShiftLeft/Right first — use `physicalShiftHeld` to tell them apart.
 */
export function resolvePhysicalPress(
	event: KeyboardEvent,
	hitMap: Map<string, BindingHit[]>,
	opts: {
		preferredLayerId: string | null;
		activeLayerId: string;
		physicalShiftHeld?: boolean;
	}
): ResolvedPress | null {
	const shiftHeld = event.shiftKey;
	const physicalShiftHeld = opts.physicalShiftHeld ?? false;
	const fromKey = EVENT_KEY_TO_ZMK[event.key];
	const fromCode = EVENT_CODE_TO_ZMK[event.code];
	const preferred = opts.preferredLayerId;

	const candidates: string[] = [];

	// Synthetic shift (ZMK symbol keycode) or explicit Lower/Raise hold → prefer layer symbol.
	const treatAsLayerSymbol =
		Boolean(fromKey) && shiftHeld && (!physicalShiftHeld || (preferred !== null && preferred !== 'base'));

	if (treatAsLayerSymbol && fromKey) {
		candidates.push(fromKey);
	}
	if (fromCode) candidates.push(fromCode);
	if (fromKey && !candidates.includes(fromKey)) candidates.push(fromKey);

	let best: BindingHit | null = null;
	let impliesLayer = false;

	for (const code of candidates) {
		const hits = hitMap.get(code) ?? [];
		if (hits.length === 0) continue;

		const layerIds = [...new Set(hits.map((hit) => hit.layerId))];
		const onlyNonBase = layerIds.length > 0 && layerIds.every((id) => id !== 'base');

		let hit: BindingHit | null = null;
		if (onlyNonBase) {
			hit = pickHit(hits, preferred, opts.activeLayerId);
			impliesLayer = true;
		} else if (preferred && preferred !== 'base') {
			hit = pickHit(hits, preferred, opts.activeLayerId);
			impliesLayer = hit?.layerId !== 'base';
		} else {
			hit = pickHit(hits, preferred, opts.activeLayerId);
			impliesLayer = Boolean(hit && hit.layerId !== 'base' && onlyNonBase);
		}

		if (hit) {
			best = hit;
			// Arrow keys / F-keys / Lower-only symbols imply their layer.
			if (hit.layerId !== 'base') {
				const baseHits = hits.filter((h) => h.layerId === 'base');
				impliesLayer = baseHits.length === 0 || treatAsLayerSymbol || onlyNonBase;
			}
			break;
		}
	}

	if (!best) return null;

	return {
		index: best.index,
		layerId: best.layerId,
		code: best.code,
		impliesLayer: impliesLayer && best.layerId !== 'base',
		shiftHeld
	};
}
