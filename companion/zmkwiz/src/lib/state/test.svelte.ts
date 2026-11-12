import { lily58 } from '#lib/boards/lily58';
import { bindingAt } from '#lib/keymap/resolve';
import type { Binding, PhysicalKey } from '#lib/keymap/types';
import { buildZmkHitMap, resolvePhysicalPress } from '#lib/typing/eventMap';

export type TestEvent = {
	at: number;
	type: 'down' | 'up';
	code: string;
	key: string;
	mappedIndex: number | null;
	label: string | null;
	layerId: string | null;
	impliesLayer: boolean;
};

export type TestTone = 'idle' | 'seen' | 'active' | 'suspect' | 'rowmate' | 'colmate';

/** Right-half Raise (`&mo 2`) on the PandaKB Lily58 map. */
export const RIGHT_RAISE_INDEX = 55;

class TestState {
	seen = $state<number[]>([]);
	held = $state<number[]>([]);
	counts = $state<Record<number, number>>({});
	suspectIndex = $state<number | null>(null);
	lastEvent = $state<TestEvent | null>(null);
	log = $state<TestEvent[]>([]);
	unmappedCount = $state(0);
	totalDowns = $state(0);
	/** Raise/Lower-only HID while debugging a momentary key. */
	layerProbeHits = $state(0);

	#hitMap = buildZmkHitMap(lily58.layers);
	#heldCodes = new Set<string>();
	#codeToIndex = new Map<string, number>();

	suspect = $derived.by((): PhysicalKey | null => {
		if (this.suspectIndex === null) return null;
		return lily58.keys[this.suspectIndex] ?? null;
	});

	suspectBinding = $derived.by((): Binding | null => {
		if (this.suspectIndex === null) return null;
		return bindingAt(lily58.layers[0]!, this.suspectIndex);
	});

	suspectLabel = $derived.by(() => {
		if (this.suspectIndex === null) return null;
		return this.suspectBinding?.label ?? `#${this.suspectIndex}`;
	});

	/** True when the marked key is a momentary layer (`&mo` / `&mo_prev`) — never emits HID alone. */
	suspectIsMomentary = $derived(
		this.suspectBinding?.behavior === 'mo' || this.suspectBinding?.behavior === 'mo_prev'
	);

	suspectTargetLayerId = $derived.by((): string | null => {
		const binding = this.suspectBinding;
		if (
			(binding?.behavior !== 'mo' && binding?.behavior !== 'mo_prev') ||
			binding.params[0] == null
		) {
			return null;
		}
		return lily58.layers[Number(binding.params[0])]?.id ?? null;
	});

	rowMates = $derived.by(() => {
		const key = this.suspect;
		if (!key) return [] as PhysicalKey[];
		return lily58.keys.filter((k) => k.row === key.row && k.index !== key.index);
	});

	colMates = $derived.by(() => {
		const key = this.suspect;
		if (!key) return [] as PhysicalKey[];
		return lily58.keys.filter((k) => k.col === key.col && k.index !== key.index);
	});

	diagnosis = $derived.by(():
		| 'unknown'
		| 'single'
		| 'row'
		| 'col'
		| 'working'
		| 'mo-probe'
		| 'mo-ok'
		| 'mo-dead' => {
		const key = this.suspect;
		if (!key) return 'unknown';

		if (this.suspectIsMomentary) {
			if (this.layerProbeHits > 0) return 'mo-ok';
			// Need at least one attempt that looked like a Raise probe (or any downs while marked).
			if (this.totalDowns === 0) return 'mo-probe';
			return 'mo-dead';
		}

		const selfSeen = this.seen.includes(key.index);
		if (selfSeen) return 'working';

		const rowDead = this.rowMates.length > 0 && this.rowMates.every((k) => !this.seen.includes(k.index));
		const colDead = this.colMates.length > 0 && this.colMates.every((k) => !this.seen.includes(k.index));
		const rowSome = this.rowMates.some((k) => this.seen.includes(k.index));
		const colSome = this.colMates.some((k) => this.seen.includes(k.index));

		const explored = this.seen.length >= 4;
		if (!explored) return 'single';
		if (rowDead && colSome) return 'row';
		if (colDead && rowSome) return 'col';
		return 'single';
	});

	toneFor(index: number): TestTone {
		if (this.suspectIndex === index) return 'suspect';
		if (this.held.includes(index)) return 'active';
		if (this.seen.includes(index)) return 'seen';
		const key = this.suspect;
		if (key) {
			const other = lily58.keys[index];
			if (other && other.row === key.row) return 'rowmate';
			if (other && other.col === key.col) return 'colmate';
		}
		return 'idle';
	}

	markSuspect(index: number | null) {
		this.suspectIndex = index;
		this.layerProbeHits = 0;
	}

	markRightRaise() {
		this.markSuspect(RIGHT_RAISE_INDEX);
	}

	reset() {
		this.seen = [];
		this.held = [];
		this.counts = {};
		this.lastEvent = null;
		this.log = [];
		this.unmappedCount = 0;
		this.totalDowns = 0;
		this.layerProbeHits = 0;
		this.#heldCodes.clear();
		this.#codeToIndex.clear();
	}

	handleKeyDown(event: KeyboardEvent) {
		if (event.repeat) return;
		const target = event.target as HTMLElement | null;
		if (
			target &&
			(target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
		) {
			return;
		}

		event.preventDefault();
		this.#heldCodes.add(event.code);
		this.totalDowns += 1;

		const physicalShiftHeld =
			this.#heldCodes.has('ShiftLeft') || this.#heldCodes.has('ShiftRight');
		const preferred = this.suspectTargetLayerId;
		const resolved = resolvePhysicalPress(event, this.#hitMap, {
			preferredLayerId: preferred,
			activeLayerId: preferred ?? 'base',
			physicalShiftHeld
		});

		const mappedIndex = resolved?.index ?? null;
		const label =
			mappedIndex !== null
				? (bindingAt(
						lily58.layers.find((l) => l.id === (resolved?.layerId ?? 'base')) ?? lily58.layers[0]!,
						mappedIndex
					)?.label ?? `#${mappedIndex}`)
				: null;

		const entry: TestEvent = {
			at: Date.now(),
			type: 'down',
			code: event.code,
			key: event.key,
			mappedIndex,
			label,
			layerId: resolved?.layerId ?? null,
			impliesLayer: resolved?.impliesLayer ?? false
		};
		this.lastEvent = entry;
		this.log = [entry, ...this.log].slice(0, 40);

		if (
			this.suspectIsMomentary &&
			resolved &&
			resolved.impliesLayer &&
			resolved.layerId === this.suspectTargetLayerId
		) {
			this.layerProbeHits += 1;
		}

		if (mappedIndex === null) {
			this.unmappedCount += 1;
			return;
		}

		this.#codeToIndex.set(event.code, mappedIndex);
		if (!this.seen.includes(mappedIndex)) {
			this.seen = [...this.seen, mappedIndex];
		}
		if (!this.held.includes(mappedIndex)) {
			this.held = [...this.held, mappedIndex];
		}
		this.counts = {
			...this.counts,
			[mappedIndex]: (this.counts[mappedIndex] ?? 0) + 1
		};
	}

	handleKeyUp(event: KeyboardEvent) {
		const target = event.target as HTMLElement | null;
		if (
			target &&
			(target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
		) {
			return;
		}

		this.#heldCodes.delete(event.code);
		const mappedIndex = this.#codeToIndex.get(event.code) ?? null;
		this.#codeToIndex.delete(event.code);

		const entry: TestEvent = {
			at: Date.now(),
			type: 'up',
			code: event.code,
			key: event.key,
			mappedIndex,
			label:
				mappedIndex !== null
					? (bindingAt(lily58.layers[0]!, mappedIndex)?.label ?? `#${mappedIndex}`)
					: null,
			layerId: null,
			impliesLayer: false
		};
		this.lastEvent = entry;
		this.log = [entry, ...this.log].slice(0, 40);

		if (mappedIndex !== null) {
			this.held = this.held.filter((i) => i !== mappedIndex);
		}
	}
}

export const testState = new TestState();
