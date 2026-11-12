/** Unshifted / shifted characters for US QWERTY keycodes used on the Lily58 base layer. */
export const KEYCODE_CHARS: Record<string, { base: string; shift: string }> = {
	A: { base: 'a', shift: 'A' },
	B: { base: 'b', shift: 'B' },
	C: { base: 'c', shift: 'C' },
	D: { base: 'd', shift: 'D' },
	E: { base: 'e', shift: 'E' },
	F: { base: 'f', shift: 'F' },
	G: { base: 'g', shift: 'G' },
	H: { base: 'h', shift: 'H' },
	I: { base: 'i', shift: 'I' },
	J: { base: 'j', shift: 'J' },
	K: { base: 'k', shift: 'K' },
	L: { base: 'l', shift: 'L' },
	M: { base: 'm', shift: 'M' },
	N: { base: 'n', shift: 'N' },
	O: { base: 'o', shift: 'O' },
	P: { base: 'p', shift: 'P' },
	Q: { base: 'q', shift: 'Q' },
	R: { base: 'r', shift: 'R' },
	S: { base: 's', shift: 'S' },
	T: { base: 't', shift: 'T' },
	U: { base: 'u', shift: 'U' },
	V: { base: 'v', shift: 'V' },
	W: { base: 'w', shift: 'W' },
	X: { base: 'x', shift: 'X' },
	Y: { base: 'y', shift: 'Y' },
	Z: { base: 'z', shift: 'Z' },
	N1: { base: '1', shift: '!' },
	N2: { base: '2', shift: '@' },
	N3: { base: '3', shift: '#' },
	N4: { base: '4', shift: '$' },
	N5: { base: '5', shift: '%' },
	N6: { base: '6', shift: '^' },
	N7: { base: '7', shift: '&' },
	N8: { base: '8', shift: '*' },
	N9: { base: '9', shift: '(' },
	N0: { base: '0', shift: ')' },
	GRAVE: { base: '`', shift: '~' },
	MINUS: { base: '-', shift: '_' },
	EQUAL: { base: '=', shift: '+' },
	LBKT: { base: '[', shift: '{' },
	RBKT: { base: ']', shift: '}' },
	BSLH: { base: '\\', shift: '|' },
	SEMI: { base: ';', shift: ':' },
	SQT: { base: "'", shift: '"' },
	COMMA: { base: ',', shift: '<' },
	DOT: { base: '.', shift: '>' },
	FSLH: { base: '/', shift: '?' },
	SPACE: { base: ' ', shift: ' ' },
	RET: { base: '\n', shift: '\n' },
	BSPC: { base: '\b', shift: '\b' },
	TAB: { base: '\t', shift: '\t' }
};

export type CharTarget = {
	index: number;
	needsShift: boolean;
	code: string;
};

export function buildCharTargets(
	bindings: { behavior: string; params: string[] }[]
): Map<string, CharTarget> {
	const map = new Map<string, CharTarget>();

	for (let index = 0; index < bindings.length; index++) {
		const binding = bindings[index];
		if (binding.behavior !== 'kp') continue;
		const code = binding.params[0];
		if (!code) continue;
		const chars = KEYCODE_CHARS[code];
		if (!chars) continue;

		if (!map.has(chars.base)) {
			map.set(chars.base, { index, needsShift: false, code });
		}
		if (chars.shift !== chars.base && !map.has(chars.shift)) {
			map.set(chars.shift, { index, needsShift: true, code });
		}
	}

	return map;
}

export function findShiftIndices(
	bindings: { behavior: string; params: string[] }[]
): number[] {
	const indices: number[] = [];
	for (let index = 0; index < bindings.length; index++) {
		const binding = bindings[index];
		if (binding.behavior === 'kp' && (binding.params[0] === 'LSHFT' || binding.params[0] === 'RSHFT')) {
			indices.push(index);
		}
	}
	return indices;
}
