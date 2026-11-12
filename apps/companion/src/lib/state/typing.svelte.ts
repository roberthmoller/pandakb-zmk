import { lily58 } from '#lib/boards/lily58';
import { buildCharTargets, findShiftIndices, type CharTarget } from '#lib/typing/keycodes';
import {
	bindMap,
	layoutById,
	pickDrill,
	type GameBind,
	type TrainerLayout,
	type TrainerLayoutId
} from '#lib/typing/layouts';

export type KeyTone = 'expected' | 'shift' | 'wrong' | 'hit' | null;

export type LastPress = {
	char: string;
	expected: string;
	correct: boolean;
	index: number | null;
	at: number;
	judgment?: 'perfect' | 'great' | 'hit' | 'miss' | 'overhit';
};

export type HighwayNoteState = 'active' | 'hit' | 'miss';

export type HighwayNote = {
	id: number;
	char: string;
	lane: number;
	index: number;
	/** 0 at spawn, 1 at freeline, >1 past the line. */
	progress: number;
	state: HighwayNoteState;
	/** Timestamp when hit/miss resolved, for FX. */
	resolvedAt: number | null;
};

export type HighwayLane = {
	char: string;
	bind: GameBind | null;
	color: string;
};

/** Time from spawn to freeline. */
export const NOTE_TRAVEL_MS = 2100;
/** Spacing between consecutive notes. */
export const NOTE_BEAT_MS = 420;
/** Lead-in before the first note spawns. */
export const NOTE_LEADIN_MS = 500;
/** Half-width of the hit window in progress units around the freeline. */
export const HIT_WINDOW = 0.14;
/** Progress past which an active note becomes a miss. */
export const MISS_PROGRESS = 1 + HIT_WINDOW;

const LANE_PALETTE = ['#22d3ee', '#f472b6', '#fbbf24', '#4ade80', '#a78bfa', '#fb7185', '#94a3b8', '#f97316'];

class TypingState {
	layoutId = $state<TrainerLayoutId>('typing');
	prompt = $state(pickDrill(layoutById('typing')));
	typed = $state('');
	startedAt = $state<number | null>(null);
	finishedAt = $state<number | null>(null);
	errors = $state(0);
	mistakeCounts = $state<Record<number, number>>({});
	wrongIndex = $state<number | null>(null);
	hitIndex = $state<number | null>(null);
	wrongUntil = $state(0);
	hitUntil = $state(0);
	lastPress = $state<LastPress | null>(null);

	/** Continuous Guitar Hero chart. */
	highwayNotes = $state<HighwayNote[]>([]);
	highwayLanes = $state<HighwayLane[]>([]);
	highwayRunning = $state(false);
	highwayNow = $state(0);
	score = $state(0);
	streak = $state(0);
	bestStreak = $state(0);
	hits = $state(0);
	misses = $state(0);
	/** 0–100 rock meter. */
	rock = $state(50);
	freelinePulseUntil = $state(0);
	laneFlash = $state<Record<number, number>>({});

	#flashTimer: ReturnType<typeof setTimeout> | null = null;
	#raf = 0;
	#chartStartAt = 0;
	#targets = buildCharTargets(lily58.layers[0]?.bindings ?? []);
	#shiftIndices = findShiftIndices(lily58.layers[0]?.bindings ?? []);

	layout = $derived.by((): TrainerLayout => layoutById(this.layoutId));
	#bindByChar = $derived.by(() => bindMap(this.layout));
	isGameLayout = $derived(this.layout.binds.length > 0);

	cursor = $derived(this.typed.length);
	expectedChar = $derived(this.prompt[this.cursor] ?? null);
	done = $derived.by(() => {
		if (this.isGameLayout) {
			return (
				this.highwayNotes.length > 0 &&
				this.highwayNotes.every((n) => n.state !== 'active') &&
				!this.highwayRunning
			);
		}
		return this.typed.length >= this.prompt.length && this.prompt.length > 0;
	});

	multiplier = $derived.by(() => {
		if (this.streak >= 30) return 4;
		if (this.streak >= 20) return 3;
		if (this.streak >= 10) return 2;
		return 1;
	});

	expectedTarget = $derived.by((): CharTarget | null => {
		if (this.isGameLayout) {
			const note = this.#windowNotes()[0];
			if (!note) return null;
			return this.#targets.get(note.char) ?? null;
		}
		if (this.expectedChar === null) return null;
		return this.#targets.get(this.expectedChar) ?? null;
	});

	expectedBind = $derived.by((): GameBind | null => {
		if (this.isGameLayout) {
			const note = this.#windowNotes()[0];
			if (!note) return null;
			return this.#bindByChar.get(note.char) ?? null;
		}
		if (this.expectedChar === null) return null;
		return this.#bindByChar.get(this.expectedChar) ?? null;
	});

	elapsedMs = $derived.by(() => {
		if (!this.startedAt) return 0;
		const end = this.finishedAt ?? Date.now();
		return Math.max(0, end - this.startedAt);
	});

	wpm = $derived.by(() => {
		const minutes = this.elapsedMs / 60000;
		if (minutes <= 0) return 0;
		const correctChars = this.isGameLayout
			? this.hits
			: [...this.typed].filter((ch, i) => ch === this.prompt[i]).length;
		return Math.round(correctChars / 5 / minutes);
	});

	accuracy = $derived.by(() => {
		if (this.isGameLayout) {
			const total = this.hits + this.misses;
			if (total === 0) return 100;
			return Math.round((this.hits / total) * 100);
		}
		const total = this.typed.length;
		if (total === 0) return 100;
		const correct = [...this.typed].filter((ch, i) => ch === this.prompt[i]).length;
		return Math.round((correct / total) * 100);
	});

	topMisses = $derived.by(() => {
		return Object.entries(this.mistakeCounts)
			.map(([index, count]) => ({ index: Number(index), count }))
			.sort((a, b) => b.count - a.count)
			.slice(0, 5);
	});

	visibleHighwayNotes = $derived.by(() => {
		const now = this.highwayNow;
		return this.highwayNotes.filter((note) => {
			if (note.state === 'active') return note.progress > -0.08 && note.progress < 1.35;
			if (note.resolvedAt == null) return false;
			return now - note.resolvedAt < 320;
		});
	});

	overlayForIndex(index: number): GameBind | null {
		if (!this.isGameLayout) return null;
		for (const [char, target] of this.#targets) {
			if (target.index === index && !target.needsShift) {
				return this.#bindByChar.get(char) ?? null;
			}
		}
		return null;
	}

	displayForChar(char: string): string {
		if (!this.isGameLayout) return char === ' ' ? '·' : char;
		const bind = this.#bindByChar.get(char);
		if (bind) return bind.label;
		return char === ' ' ? '·' : char;
	}

	bindForChar(char: string): GameBind | null {
		return this.#bindByChar.get(char) ?? null;
	}

	toneFor(index: number): KeyTone {
		if (this.wrongIndex === index && Date.now() < this.wrongUntil) return 'wrong';
		if (this.hitIndex === index && Date.now() < this.hitUntil) return 'hit';
		const target = this.expectedTarget;
		if (!target || this.done) return null;
		if (target.index === index) return 'expected';
		if (target.needsShift && this.#shiftIndices.includes(index)) return 'shift';
		return null;
	}

	setLayout(id: TrainerLayoutId) {
		if (this.layoutId === id) return;
		this.layoutId = id;
		this.reset(pickDrill(layoutById(id)));
	}

	reset(nextPrompt?: string) {
		this.#stopChart();
		if (this.#flashTimer) clearTimeout(this.#flashTimer);
		this.prompt = nextPrompt ?? pickDrill(this.layout, this.prompt);
		this.typed = '';
		this.startedAt = null;
		this.finishedAt = null;
		this.errors = 0;
		this.mistakeCounts = {};
		this.wrongIndex = null;
		this.hitIndex = null;
		this.wrongUntil = 0;
		this.hitUntil = 0;
		this.lastPress = null;
		this.score = 0;
		this.streak = 0;
		this.bestStreak = 0;
		this.hits = 0;
		this.misses = 0;
		this.rock = 50;
		this.freelinePulseUntil = 0;
		this.laneFlash = {};
		this.highwayNotes = [];
		this.highwayLanes = [];
		this.highwayNow = 0;
		if (this.isGameLayout && this.prompt.length > 0) {
			this.#bootChart();
		}
	}

	#stopChart() {
		this.highwayRunning = false;
		if (this.#raf) {
			cancelAnimationFrame(this.#raf);
			this.#raf = 0;
		}
	}

	#bootChart() {
		const chars = [...this.prompt];
		const unique = [...new Set(chars)];
		// Left→right by physical Lily58 key position so frets mirror the board.
		unique.sort((a, b) => {
			const ia = this.#targets.get(a)?.index ?? 999;
			const ib = this.#targets.get(b)?.index ?? 999;
			const xa = lily58.keys[ia]?.x ?? 999;
			const xb = lily58.keys[ib]?.x ?? 999;
			return xa - xb || ia - ib;
		});
		this.highwayLanes = unique.map((char, lane) => {
			const bind = this.#bindByChar.get(char) ?? null;
			return {
				char,
				bind,
				color: bind?.color ?? LANE_PALETTE[lane % LANE_PALETTE.length]!
			};
		});
		const laneOf = new Map(unique.map((char, lane) => [char, lane]));
		this.highwayNotes = chars.map((char, index) => ({
			id: index,
			char,
			lane: laneOf.get(char) ?? 0,
			index,
			progress: -1,
			state: 'active' as const,
			resolvedAt: null
		}));
		this.#chartStartAt = performance.now() + NOTE_LEADIN_MS;
		this.highwayRunning = true;
		this.highwayNow = performance.now();
		const loop = (t: number) => {
			this.#tick(t);
			if (this.highwayRunning) this.#raf = requestAnimationFrame(loop);
		};
		this.#raf = requestAnimationFrame(loop);
	}

	#tick(now: number) {
		this.highwayNow = now;
		if (!this.highwayRunning) return;

		const chartTime = now - this.#chartStartAt;
		let unresolved = 0;
		const next = this.highwayNotes.map((note) => {
			if (note.state !== 'active') return note;
			const spawnAt = note.index * NOTE_BEAT_MS;
			const progress = (chartTime - spawnAt) / NOTE_TRAVEL_MS;
			if (progress >= MISS_PROGRESS) {
				this.#registerMiss(note);
				this.lastPress = {
					char: note.char === ' ' ? '␣' : note.char,
					expected: note.char === ' ' ? '␣' : note.char,
					correct: false,
					index: this.#targets.get(note.char)?.index ?? null,
					at: Date.now(),
					judgment: 'miss'
				};
				return {
					...note,
					progress,
					state: 'miss' as const,
					resolvedAt: now
				};
			}
			unresolved += 1;
			return { ...note, progress };
		});
		this.highwayNotes = next;

		if (unresolved === 0 && chartTime > 0) {
			this.highwayRunning = false;
			this.finishedAt = Date.now();
			if (this.#raf) {
				cancelAnimationFrame(this.#raf);
				this.#raf = 0;
			}
		}
	}

	#windowNotes(): HighwayNote[] {
		return this.highwayNotes
			.filter(
				(n) => n.state === 'active' && Math.abs(n.progress - 1) <= HIT_WINDOW
			)
			.sort((a, b) => a.progress - b.progress);
	}

	#registerMiss(note: HighwayNote) {
		this.misses += 1;
		this.errors += 1;
		this.streak = 0;
		this.rock = Math.max(0, this.rock - 8);
		const target = this.#targets.get(note.char);
		if (target) {
			this.mistakeCounts = {
				...this.mistakeCounts,
				[target.index]: (this.mistakeCounts[target.index] ?? 0) + 1
			};
			this.flash(target.index, false);
		}
	}

	#judgmentFor(progress: number): 'perfect' | 'great' | 'hit' {
		const delta = Math.abs(progress - 1);
		if (delta <= 0.045) return 'perfect';
		if (delta <= 0.09) return 'great';
		return 'hit';
	}

	handleKeydown(event: KeyboardEvent) {
		if (this.done) {
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault();
				this.reset();
			}
			return;
		}

		if (event.key === 'Backspace') {
			if (this.isGameLayout) {
				event.preventDefault();
				return;
			}
			event.preventDefault();
			this.typed = this.typed.slice(0, -1);
			this.finishedAt = null;
			this.lastPress = {
				char: '⌫',
				expected: this.prompt[this.typed.length] ?? '',
				correct: true,
				index: this.#targets.get('\b')?.index ?? null,
				at: Date.now()
			};
			return;
		}

		if (event.key === 'Escape') {
			event.preventDefault();
			this.reset(this.prompt);
			return;
		}

		if (event.key.length !== 1 || event.ctrlKey || event.metaKey || event.altKey) {
			return;
		}

		event.preventDefault();

		if (this.isGameLayout) {
			this.#handleGameKey(event.key);
			return;
		}

		if (!this.startedAt) this.startedAt = Date.now();

		const expected = this.prompt[this.typed.length] ?? '';
		const typedChar = event.key;
		const expectedTarget = expected ? this.#targets.get(expected) : null;
		const typedTarget = this.#targets.get(typedChar);
		const correct = typedChar === expected;

		this.lastPress = {
			char: typedChar === ' ' ? '␣' : typedChar,
			expected: expected === ' ' ? '␣' : expected,
			correct,
			index: typedTarget?.index ?? expectedTarget?.index ?? null,
			at: Date.now()
		};

		if (!correct) {
			this.errors += 1;
			const missIndex = typedTarget?.index ?? expectedTarget?.index;
			if (missIndex !== undefined) {
				this.mistakeCounts = {
					...this.mistakeCounts,
					[missIndex]: (this.mistakeCounts[missIndex] ?? 0) + 1
				};
			}
			this.flash(typedTarget?.index ?? expectedTarget?.index ?? null, false);
		} else {
			this.flash(typedTarget?.index ?? expectedTarget?.index ?? null, true);
		}

		this.typed += typedChar;

		if (this.typed.length >= this.prompt.length) {
			this.finishedAt = Date.now();
		}
	}

	#handleGameKey(typedChar: string) {
		if (!this.startedAt) this.startedAt = Date.now();

		const typedTarget = this.#targets.get(typedChar);
		const candidates = this.highwayNotes
			.filter(
				(n) =>
					n.state === 'active' &&
					n.char === typedChar &&
					Math.abs(n.progress - 1) <= HIT_WINDOW
			)
			.sort((a, b) => Math.abs(a.progress - 1) - Math.abs(b.progress - 1));

		const note = candidates[0];
		if (!note) {
			// Overhit — pressed with nothing in the window for that key.
			this.misses += 1;
			this.errors += 1;
			this.streak = 0;
			this.rock = Math.max(0, this.rock - 5);
			this.lastPress = {
				char: typedChar === ' ' ? '␣' : typedChar,
				expected: '',
				correct: false,
				index: typedTarget?.index ?? null,
				at: Date.now(),
				judgment: 'overhit'
			};
			this.flash(typedTarget?.index ?? null, false);
			return;
		}

		const judgment = this.#judgmentFor(note.progress);
		const points = (judgment === 'perfect' ? 100 : judgment === 'great' ? 75 : 50) * this.multiplier;

		this.highwayNotes = this.highwayNotes.map((n) =>
			n.id === note.id
				? { ...n, state: 'hit' as const, resolvedAt: this.highwayNow, progress: 1 }
				: n
		);
		this.hits += 1;
		this.typed += typedChar;
		this.streak += 1;
		this.bestStreak = Math.max(this.bestStreak, this.streak);
		this.score += points;
		this.rock = Math.min(100, this.rock + (judgment === 'perfect' ? 6 : 4));
		this.freelinePulseUntil = this.highwayNow + 180;
		this.laneFlash = { ...this.laneFlash, [note.lane]: this.highwayNow + 220 };

		this.lastPress = {
			char: typedChar === ' ' ? '␣' : typedChar,
			expected: typedChar === ' ' ? '␣' : typedChar,
			correct: true,
			index: typedTarget?.index ?? null,
			at: Date.now(),
			judgment
		};
		this.flash(typedTarget?.index ?? null, true);
	}

	flash(index: number | null, correct: boolean) {
		if (this.#flashTimer) clearTimeout(this.#flashTimer);
		if (correct) {
			this.hitIndex = index;
			this.hitUntil = Date.now() + 220;
			this.wrongIndex = null;
			this.wrongUntil = 0;
		} else {
			this.wrongIndex = index;
			this.wrongUntil = Date.now() + 420;
			this.hitIndex = null;
			this.hitUntil = 0;
		}
		this.#flashTimer = setTimeout(() => {
			this.wrongIndex = null;
			this.hitIndex = null;
			this.wrongUntil = 0;
			this.hitUntil = 0;
		}, correct ? 220 : 420);
	}
}

export const typingState = new TypingState();
