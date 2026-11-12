/** Sentences using characters available on the Lily58 base layer (US QWERTY). */
export const PROMPTS = [
	'the quick brown fox jumps over the lazy dog',
	'pack my box with five dozen liquor jugs',
	'how vexingly quick daft zebras jump',
	'sphinx of black quartz judge my vow',
	'a wizard types keymap layers for fun',
	'lily fifty eight feels natural after practice',
	'hold lower for symbols; stock L2 is adjust not raise',
	'bluetooth profile one connects in seconds',
	'type slowly first, then trust the muscle memory',
	'left thumb space, right thumb enter',
	'home row anchors: a s d f and j k l',
	'numbers live on the top row: 1 2 3 4 5 6 7 8 9 0',
	'comma, period, and slash sit under your right hand',
	'escape is the outer corner; backtick sits beside zero',
	'practice makes the split layout feel smaller'
];

export function pickPrompt(exclude?: string): string {
	const pool = exclude ? PROMPTS.filter((p) => p !== exclude) : PROMPTS;
	const list = pool.length > 0 ? pool : PROMPTS;
	return list[Math.floor(Math.random() * list.length)] ?? PROMPTS[0];
}
