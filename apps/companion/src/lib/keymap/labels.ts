const CODE_LABELS: Record<string, string> = {
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
	LEFT_ARROW: '←',
	RIGHT_ARROW: '→',
	UP_ARROW: '↑',
	DOWN_ARROW: '↓',
	PG_UP: 'PgUp',
	PG_DN: 'PgDn',
	HOME: 'Home',
	END: 'End',
	C_MUTE: 'Mute',
	K_MUTE: 'Mute',
	C_VOL_UP: 'Vol+',
	C_VOL_DN: 'Vol-',
	C_PP: 'Play',
	C_PAUSE: 'Pause',
	C_PREVIOUS: 'Prev',
	C_NEXT: 'Next'
};

const BEHAVIOR_GLOSS: Record<string, string> = {
	kp: 'Key press — sends a HID keycode while held.',
	mo: 'Momentary layer — holds another layer while pressed.',
	mo_prev:
		'Momentary layer with companion preview — double-tap-hold shows the layout overlay while held.',
	to: 'To layer — switches to a layer permanently.',
	tog: 'Toggle layer — flips a layer on/off.',
	lt: 'Layer-tap — momentary layer on hold, key on tap.',
	mt: 'Mod-tap — modifier on hold, key on tap.',
	bt: 'Bluetooth — clear or select a wireless profile.',
	trans: 'Transparent — falls through to the next lower layer.',
	none: 'No-op — this position does nothing.',
	rgb_ug: 'RGB underglow control.',
	ext_power: 'External power — toggles power to peripherals like the display.'
};

function digitLabel(code: string): string | null {
	const n = /^N(\d)$/.exec(code);
	if (n) return n[1];
	const f = /^F(\d{1,2})$/.exec(code);
	if (f) return `F${f[1]}`;
	return null;
}

export function labelForCode(code: string): string {
	return CODE_LABELS[code] ?? digitLabel(code) ?? code;
}

export function glossForBehavior(behavior: string): string {
	return BEHAVIOR_GLOSS[behavior] ?? `ZMK behavior &${behavior}.`;
}

export function parseBinding(raw: string): {
	behavior: string;
	params: string[];
	label: string;
	gloss: string;
} {
	const trimmed = raw.trim();
	if (trimmed === '&trans') {
		return { behavior: 'trans', params: [], label: '▽', gloss: glossForBehavior('trans') };
	}
	if (trimmed === '&none') {
		return { behavior: 'none', params: [], label: '', gloss: glossForBehavior('none') };
	}

	const parts = trimmed.replace(/^&/, '').split(/\s+/);
	const behavior = parts[0] ?? 'kp';
	const params = parts.slice(1);

	if (behavior === 'kp') {
		const code = params[0] ?? '';
		return {
			behavior,
			params,
			label: labelForCode(code),
			gloss: glossForBehavior(behavior)
		};
	}

	if (behavior === 'mo' || behavior === 'mo_prev') {
		return {
			behavior,
			params,
			label: `L${params[0] ?? '?'}`,
			gloss: `${glossForBehavior(behavior)} Target layer ${params[0] ?? '?'}.`
		};
	}

	if (behavior === 'to' || behavior === 'tog') {
		const prefix = behavior === 'to' ? 'TO' : 'TG';
		return {
			behavior,
			params,
			label: `${prefix}${params[0] ?? '?'}`,
			gloss: `${glossForBehavior(behavior)} Target layer ${params[0] ?? '?'}.`
		};
	}

	if (behavior === 'bt') {
		const cmd = params[0] ?? '';
		if (cmd === 'BT_CLR') {
			return { behavior, params, label: 'BT CLR', gloss: 'Clears Bluetooth pairings.' };
		}
		if (cmd === 'BT_SEL') {
			const profile = params[1] ?? '?';
			return {
				behavior,
				params,
				label: `BT${Number(profile) + 1}`,
				gloss: `Select Bluetooth profile ${Number(profile) + 1}.`
			};
		}
		return { behavior, params, label: 'BT', gloss: glossForBehavior(behavior) };
	}

	if (behavior === 'rgb_ug') {
		const map: Record<string, string> = {
			RGB_TOG: 'RGB',
			RGB_EFF: 'Eff',
			RGB_HUD: 'H−',
			RGB_HUI: 'H+',
			RGB_SAD: 'S−',
			RGB_SAI: 'S+',
			RGB_BRD: 'B−',
			RGB_BRI: 'B+'
		};
		const cmd = params[0] ?? '';
		return {
			behavior,
			params,
			label: map[cmd] ?? 'RGB',
			gloss: glossForBehavior(behavior)
		};
	}

	if (behavior === 'ext_power') {
		const map: Record<string, string> = {
			EP_ON: 'PWR ON',
			EP_OFF: 'PWR OFF',
			EP_TOG: 'PWR'
		};
		const cmd = params[0] ?? '';
		return {
			behavior,
			params,
			label: map[cmd] ?? 'PWR',
			gloss: glossForBehavior(behavior)
		};
	}

	return {
		behavior,
		params,
		label: params[0] ? labelForCode(params[0]) : behavior,
		gloss: glossForBehavior(behavior)
	};
}

export function bindingFromRaw(raw: string) {
	const parsed = parseBinding(raw);
	return { raw, ...parsed };
}
