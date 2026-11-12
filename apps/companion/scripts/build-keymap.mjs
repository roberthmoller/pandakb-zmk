import { writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const CODE_LABELS = {
	ESC: 'Esc',
	TAB: 'Tab',
	BSPC: 'Bksp',
	DEL: 'Del',
	RET: 'Enter',
	SPACE: 'Space',
	LSHFT: 'Shift',
	RSHFT: 'Shift',
	LCTRL: 'Ctrl',
	RCTRL: 'Ctrl',
	LALT: 'Alt',
	RALT: 'Alt',
	LGUI: 'GUI',
	RGUI: 'GUI',
	SEMI: ';',
	SQT: "'",
	GRAVE: '`',
	MINUS: '-',
	EQUAL: '=',
	PLUS: '+',
	LBKT: '[',
	RBKT: ']',
	LBRC: '{',
	RBRC: '}',
	BSLH: '\\',
	PIPE: '|',
	COMMA: ',',
	DOT: '.',
	FSLH: '/',
	TILDE: '~',
	EXCL: '!',
	AT: '@',
	HASH: '#',
	DOLLAR: '$',
	PRCNT: '%',
	CARET: '^',
	AMPS: '&',
	ASTRK: '*',
	KP_MULTIPLY: '*',
	KP_PLUS: '+',
	LPAR: '(',
	RPAR: ')',
	LEFT: '←',
	RIGHT: '→',
	UP: '↑',
	DOWN: '↓',
	PG_UP: 'PgUp',
	PG_DN: 'PgDn',
	C_MUTE: 'Mute'
};

const GLOSS = {
	kp: 'Key press — sends a HID keycode while held.',
	mo: 'Momentary layer — holds another layer while pressed.',
	bt: 'Bluetooth — clear or select a wireless profile.',
	trans: 'Transparent — falls through to the next lower layer.',
	rgb_ug: 'RGB underglow control.',
	ext_power: 'External power — toggles power to peripherals like the display.'
};

function labelForCode(code) {
	if (CODE_LABELS[code]) return CODE_LABELS[code];
	const n = /^N(\d)$/.exec(code);
	if (n) return n[1];
	const f = /^F(\d{1,2})$/.exec(code);
	if (f) return `F${f[1]}`;
	return code;
}

function bindingFromRaw(raw) {
	const trimmed = raw.trim();
	if (trimmed === '&trans') {
		return { raw: trimmed, behavior: 'trans', params: [], label: '▽', gloss: GLOSS.trans };
	}
	const parts = trimmed.replace(/^&/, '').split(/\s+/);
	const behavior = parts[0];
	const params = parts.slice(1);

	if (behavior === 'kp') {
		return {
			raw: trimmed,
			behavior,
			params,
			label: labelForCode(params[0] ?? ''),
			gloss: GLOSS.kp
		};
	}
	if (behavior === 'mo') {
		return {
			raw: trimmed,
			behavior,
			params,
			label: `L${params[0]}`,
			gloss: `${GLOSS.mo} Target layer ${params[0]}.`
		};
	}
	if (behavior === 'bt') {
		if (params[0] === 'BT_CLR') {
			return { raw: trimmed, behavior, params, label: 'BT CLR', gloss: 'Clears Bluetooth pairings.' };
		}
		if (params[0] === 'BT_SEL') {
			return {
				raw: trimmed,
				behavior,
				params,
				label: `BT${Number(params[1]) + 1}`,
				gloss: `Select Bluetooth profile ${Number(params[1]) + 1}.`
			};
		}
	}
	if (behavior === 'rgb_ug') {
		const map = {
			RGB_TOG: 'RGB',
			RGB_EFF: 'Eff',
			RGB_HUD: 'H−',
			RGB_HUI: 'H+',
			RGB_SAD: 'S−',
			RGB_SAI: 'S+',
			RGB_BRD: 'B−',
			RGB_BRI: 'B+'
		};
		return {
			raw: trimmed,
			behavior,
			params,
			label: map[params[0]] ?? 'RGB',
			gloss: GLOSS.rgb_ug
		};
	}
	if (behavior === 'ext_power') {
		const map = { EP_ON: 'PWR ON', EP_OFF: 'PWR OFF', EP_TOG: 'PWR' };
		return {
			raw: trimmed,
			behavior,
			params,
			label: map[params[0]] ?? 'PWR',
			gloss: GLOSS.ext_power
		};
	}
	return {
		raw: trimmed,
		behavior,
		params,
		label: params[0] ? labelForCode(params[0]) : behavior,
		gloss: GLOSS[behavior] ?? `ZMK behavior &${behavior}.`
	};
}

function layer(id, name, raws) {
	if (raws.length !== 58) {
		throw new Error(`Layer ${id} has ${raws.length} bindings, expected 58`);
	}
	return { id, name, bindings: raws.map(bindingFromRaw) };
}

/** Exact bindings from PandaKB zmk-for-lily `config/lily58.keymap` (shipped). */
const base = [
	'&kp ESC', '&kp N1', '&kp N2', '&kp N3', '&kp N4', '&kp N5', '&kp N6', '&kp N7', '&kp N8', '&kp N9', '&kp N0', '&kp GRAVE',
	'&kp TAB', '&kp Q', '&kp W', '&kp E', '&kp R', '&kp T', '&kp Y', '&kp U', '&kp I', '&kp O', '&kp P', '&kp MINUS',
	'&kp LCTRL', '&kp A', '&kp S', '&kp D', '&kp F', '&kp G', '&kp H', '&kp J', '&kp K', '&kp L', '&kp SEMI', '&kp SQT',
	'&kp LSHFT', '&kp Z', '&kp X', '&kp C', '&kp V', '&kp B', '&kp C_MUTE', '&rgb_ug RGB_TOG', '&kp N', '&kp M', '&kp COMMA', '&kp DOT', '&kp FSLH', '&kp RSHFT',
	'&kp LALT', '&kp LGUI', '&mo 1', '&kp SPACE', '&kp RET', '&mo 2', '&kp BSPC', '&kp RGUI'
];

const lower = [
	'&bt BT_CLR', '&bt BT_SEL 0', '&bt BT_SEL 1', '&bt BT_SEL 2', '&bt BT_SEL 3', '&bt BT_SEL 4', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&kp F1', '&kp F2', '&kp F3', '&kp F4', '&kp F5', '&kp F6', '&kp F7', '&kp F8', '&kp F9', '&kp F10', '&kp F11', '&kp F12',
	'&kp GRAVE', '&kp EXCL', '&kp AT', '&kp HASH', '&kp DOLLAR', '&kp PRCNT', '&kp CARET', '&kp AMPS', '&kp ASTRK', '&kp LPAR', '&kp RPAR', '&kp TILDE',
	'&trans', '&ext_power EP_ON', '&ext_power EP_OFF', '&ext_power EP_TOG', '&trans', '&trans', '&trans', '&trans', '&trans', '&kp MINUS', '&kp PLUS', '&kp LBRC', '&kp RBRC', '&kp PIPE',
	'&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans'
];

/**
 * As flashed: there is no Raise. `&mo 2` activates this Adjust layer (index 2).
 * Right-hand alphas are `&trans`, so L2+J still types j — that is expected on stock PandaKB.
 * conditional_layers wants Adjust on layer 3 when Lower+Raise are held, but Raise was never defined.
 */
const adjust = [
	'&bt BT_CLR', '&bt BT_SEL 0', '&bt BT_SEL 1', '&bt BT_SEL 2', '&bt BT_SEL 3', '&bt BT_SEL 4', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&ext_power EP_TOG', '&rgb_ug RGB_HUD', '&rgb_ug RGB_HUI', '&rgb_ug RGB_SAD', '&rgb_ug RGB_SAI', '&rgb_ug RGB_EFF', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&trans', '&rgb_ug RGB_BRD', '&rgb_ug RGB_BRI', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans'
];

/**
 * Intended Raise (not in upstream file). Kept here so the site can show a "fixed firmware"
 * preview; `layers` below uses shipped order unless FIXED=1.
 */
const raise = [
	'&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans',
	'&kp GRAVE', '&kp N1', '&kp N2', '&kp N3', '&kp N4', '&kp N5', '&kp N6', '&kp N7', '&kp N8', '&kp N9', '&kp N0', '&trans',
	'&kp F1', '&kp F2', '&kp F3', '&kp F4', '&kp F5', '&kp F6', '&trans', '&kp LEFT', '&kp DOWN', '&kp UP', '&kp RIGHT', '&trans',
	'&kp F7', '&kp F8', '&kp F9', '&kp F10', '&kp F11', '&kp F12', '&trans', '&trans', '&kp PLUS', '&kp MINUS', '&kp EQUAL', '&kp LBKT', '&kp RBKT', '&kp BSLH',
	'&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans', '&trans'
];

const useFixed = process.env.KEYMAP_FIXED === '1';

const keymap = useFixed
	? {
			source: 'https://github.com/PandaKBLab/zmk-for-keyboards/tree/zmk-for-lily',
			notes:
				'Fixed layout: Base / Lower / Raise / Adjust. Raise restored so `&mo 2` is a real upper layer; Adjust moves to layer 3 for conditional_layers.',
			layers: [
				layer('base', 'Base', base),
				layer('lower', 'Lower', lower),
				layer('raise', 'Raise', raise),
				layer('adjust', 'Adjust', adjust)
			]
		}
	: {
			source: 'https://github.com/PandaKBLab/zmk-for-keyboards/tree/zmk-for-lily',
			notes:
				'Matches shipped PandaKB lily58.keymap: Base, Lower, Adjust at index 2. There is no Raise — `&mo 2` opens Adjust (mostly transparent on the right half), so L2+J types j.',
			layers: [
				layer('base', 'Base', base),
				layer('lower', 'Lower', lower),
				layer('adjust', 'Adjust', adjust)
			]
		};

const out = join(dirname(fileURLToPath(import.meta.url)), '../src/lib/boards/lily58/keymap.json');
writeFileSync(out, JSON.stringify(keymap, null, 2) + '\n');
console.log(`Wrote ${out} (${useFixed ? 'fixed' : 'shipped'} layout, ${keymap.layers.length} layers)`);
