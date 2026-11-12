import { PROMPTS } from '#lib/typing/prompts';

/** Abstract icon glyph for the highway / overlays (not official game art). */
export type SpellSymbol =
	| 'gem'
	| 'arrow-up'
	| 'arrow-down'
	| 'arrow-left'
	| 'arrow-right'
	| 'strafe-left'
	| 'strafe-right'
	| 'jump'
	| 'hand'
	| 'swords'
	| 'chat'
	| 'sit'
	| 'sheath'
	| 'bag'
	| 'person'
	| 'book'
	| 'star'
	| 'map'
	| 'scroll'
	| 'users'
	| 'eye'
	| 'ability'
	| 'ult'
	| 'summoner'
	| 'potion'
	| 'trinket'
	| 'portal'
	| 'crosshair'
	| 'stop'
	| 'camera'
	| 'lock'
	| 'shop'
	| 'chart'
	| 'ping';

/** A single default-game bind the trainer can overlay and drill. */
export type GameBind = {
	/** Character the host types (`event.key`). */
	char: string;
	/** Short label for the keycap overlay. */
	label: string;
	/** Longer name for the prompt strip. */
	action: string;
	/** Highway spell-icon glyph. */
	symbol: SpellSymbol;
	/** Accent color for the icon gem. */
	color: string;
};

export type TrainerLayoutId = 'typing' | 'wow' | 'lol';

export type TrainerLayout = {
	id: TrainerLayoutId;
	name: string;
	description: string;
	/** Empty for plain typing — board keeps keymap labels. */
	binds: GameBind[];
	drills: string[];
};

const bar = (n: number, char: string): GameBind => ({
	char,
	label: `Bar${n}`,
	action: `Action ${n}`,
	symbol: 'gem',
	color: n <= 4 ? '#c9a227' : n <= 8 ? '#3d8bfd' : '#c084fc'
});

/** World of Warcraft — stock PC defaults (combat / movement / bars). */
const WOW_BINDS: GameBind[] = [
	bar(1, '1'),
	bar(2, '2'),
	bar(3, '3'),
	bar(4, '4'),
	bar(5, '5'),
	bar(6, '6'),
	bar(7, '7'),
	bar(8, '8'),
	bar(9, '9'),
	bar(10, '0'),
	bar(11, '-'),
	{ char: 'w', label: 'Fwd', action: 'Move Forward', symbol: 'arrow-up', color: '#7dd3a8' },
	{ char: 'a', label: 'TurnL', action: 'Turn Left', symbol: 'arrow-left', color: '#7dd3a8' },
	{ char: 's', label: 'Back', action: 'Move Backward', symbol: 'arrow-down', color: '#7dd3a8' },
	{ char: 'd', label: 'TurnR', action: 'Turn Right', symbol: 'arrow-right', color: '#7dd3a8' },
	{ char: 'q', label: 'StrL', action: 'Strafe Left', symbol: 'strafe-left', color: '#5eead4' },
	{ char: 'e', label: 'StrR', action: 'Strafe Right', symbol: 'strafe-right', color: '#5eead4' },
	{ char: ' ', label: 'Jump', action: 'Jump', symbol: 'jump', color: '#fde68a' },
	{ char: 'f', label: 'Use', action: 'Interact', symbol: 'hand', color: '#fdba74' },
	{ char: 't', label: 'Atk', action: 'Attack Target', symbol: 'swords', color: '#f87171' },
	{ char: 'r', label: 'Reply', action: 'Reply', symbol: 'chat', color: '#93c5fd' },
	{ char: 'x', label: 'Sit', action: 'Sit / Stand', symbol: 'sit', color: '#a3a3a3' },
	{ char: 'z', label: 'Sheath', action: 'Sheath Weapon', symbol: 'sheath', color: '#a8a29e' },
	{ char: 'b', label: 'Bags', action: 'Open Bags', symbol: 'bag', color: '#d6b35a' },
	{ char: 'c', label: 'Char', action: 'Character', symbol: 'person', color: '#e7e5e4' },
	{ char: 'p', label: 'Spell', action: 'Spellbook', symbol: 'book', color: '#a78bfa' },
	{ char: 'n', label: 'Talent', action: 'Talents', symbol: 'star', color: '#fbbf24' },
	{ char: 'm', label: 'Map', action: 'World Map', symbol: 'map', color: '#34d399' },
	{ char: 'l', label: 'Quest', action: 'Quest Log', symbol: 'scroll', color: '#fcd34d' },
	{ char: 'o', label: 'Social', action: 'Social', symbol: 'users', color: '#60a5fa' },
	{ char: 'v', label: 'Names', action: 'Show Names', symbol: 'eye', color: '#c4b5fd' }
];

/** League of Legends — stock PC defaults. */
const LOL_BINDS: GameBind[] = [
	{ char: 'q', label: 'Q', action: 'Ability Q', symbol: 'ability', color: '#38bdf8' },
	{ char: 'w', label: 'W', action: 'Ability W', symbol: 'ability', color: '#2dd4bf' },
	{ char: 'e', label: 'E', action: 'Ability E', symbol: 'ability', color: '#a78bfa' },
	{ char: 'r', label: 'R', action: 'Ultimate', symbol: 'ult', color: '#fbbf24' },
	{ char: 'd', label: 'D', action: 'Summoner D', symbol: 'summoner', color: '#fb7185' },
	{ char: 'f', label: 'F', action: 'Summoner F', symbol: 'summoner', color: '#f472b6' },
	{ char: '1', label: 'I1', action: 'Item 1', symbol: 'potion', color: '#94a3b8' },
	{ char: '2', label: 'I2', action: 'Item 2', symbol: 'potion', color: '#94a3b8' },
	{ char: '3', label: 'I3', action: 'Item 3', symbol: 'potion', color: '#94a3b8' },
	{ char: '4', label: 'I4', action: 'Item 4', symbol: 'potion', color: '#94a3b8' },
	{ char: '5', label: 'I5', action: 'Item 5', symbol: 'potion', color: '#94a3b8' },
	{ char: '6', label: 'I6', action: 'Item 6', symbol: 'potion', color: '#94a3b8' },
	{ char: '7', label: 'Trink', action: 'Trinket', symbol: 'trinket', color: '#67e8f9' },
	{ char: 'b', label: 'Recall', action: 'Recall', symbol: 'portal', color: '#818cf8' },
	{ char: 'a', label: 'AMove', action: 'Attack Move', symbol: 'crosshair', color: '#f87171' },
	{ char: 's', label: 'Stop', action: 'Stop', symbol: 'stop', color: '#e5e5e5' },
	{ char: ' ', label: 'Cam', action: 'Center Camera', symbol: 'camera', color: '#a3a3a3' },
	{ char: 'y', label: 'Lock', action: 'Camera Lock', symbol: 'lock', color: '#facc15' },
	{ char: 'p', label: 'Shop', action: 'Shop', symbol: 'shop', color: '#fde68a' },
	{ char: 'c', label: 'Stats', action: 'Champion Stats', symbol: 'chart', color: '#86efac' },
	{ char: 'v', label: 'AClk', action: 'Attack Move Click', symbol: 'crosshair', color: '#fb923c' },
	{ char: 'x', label: 'Ping', action: 'Generic Ping', symbol: 'ping', color: '#38bdf8' }
];

/** Keep ≤5 unique keys per drill so every fret stays on-screen. */
const WOW_DRILLS = [
	'wasdwasd',
	'qeqeqe',
	'1234512345',
	'12312312',
	'ffff',
	'tttt',
	'12qwe',
	'wasd1',
	'qwasd',
	'12345',
	'bmcp',
	'zxft',
	'wasdq',
	'555666',
	'qe123',
	'asdf'
];

const LOL_DRILLS = [
	'qwer',
	'qwerqwer',
	'dfdfdf',
	'qwerd',
	'12345',
	'qqwwee',
	'qwedf',
	'bbbb',
	'asdf',
	'qwer1',
	'123qw',
	'asas',
	'qwerb',
	'ffffd',
	'pbyb',
	'd f d f',
	'q w e r'
];

export const TRAINER_LAYOUTS: TrainerLayout[] = [
	{
		id: 'typing',
		name: 'Typing',
		description: 'Sentences on the Lily58 base layer.',
		binds: [],
		drills: PROMPTS
	},
	{
		id: 'wow',
		name: 'WoW',
		description: 'Continuous highway — hit default WoW binds in the freeline window.',
		binds: WOW_BINDS,
		drills: WOW_DRILLS
	},
	{
		id: 'lol',
		name: 'LoL',
		description: 'Continuous highway — hit default LoL binds in the freeline window.',
		binds: LOL_BINDS,
		drills: LOL_DRILLS
	}
];

export function layoutById(id: TrainerLayoutId): TrainerLayout {
	return TRAINER_LAYOUTS.find((layout) => layout.id === id) ?? TRAINER_LAYOUTS[0]!;
}

export function bindForChar(layout: TrainerLayout, char: string): GameBind | null {
	return layout.binds.find((bind) => bind.char === char) ?? null;
}

export function bindMap(layout: TrainerLayout): Map<string, GameBind> {
	return new Map(layout.binds.map((bind) => [bind.char, bind]));
}

export function pickDrill(layout: TrainerLayout, exclude?: string): string {
	const pool = exclude ? layout.drills.filter((d) => d !== exclude) : layout.drills;
	const list = pool.length > 0 ? pool : layout.drills;
	return list[Math.floor(Math.random() * list.length)] ?? layout.drills[0] ?? '';
}
