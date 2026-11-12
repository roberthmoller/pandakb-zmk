import { lily58 } from '#lib/boards/lily58';
import { isMomentaryLayer } from '#lib/keymap/momentary';
import { bindingAt, layerById } from '#lib/keymap/resolve';
import {
	buildZmkHitMap,
	resolvePhysicalPress,
	type ResolvedPress
} from '#lib/typing/eventMap';
import type { Binding, BoardDefinition, Layer } from '#lib/keymap/types';

export type ConfigOrigin =
	| { kind: 'builtin'; label: string }
	| { kind: 'firmware'; label: string; path: string }
	| { kind: 'github'; label: string; url: string }
	| { kind: 'upload'; label: string; fileName: string };

class BoardState {
	board = $state<BoardDefinition>(structuredClone(lily58));
	activeLayerId = $state(lily58.layers[0]?.id ?? 'base');
	holdLayerId = $state<string | null>(null);
	selectedKeyIndex = $state<number | null>(null);
	hoveredKeyIndex = $state<number | null>(null);
	pressedIndices = $state<number[]>([]);
	/** Last resolved physical press — used by Learn stage copy. */
	lastPhysical = $state<ResolvedPress | null>(null);
	configOrigin = $state<ConfigOrigin>({ kind: 'builtin', label: 'PandaKB stock' });
	configWarnings = $state<string[]>([]);

	#hitMap = buildZmkHitMap(lily58.layers);
	#heldCodes = new Set<string>();
	/** event.code → board key index for the current press */
	#codeToIndex = new Map<string, number>();
	/** Layers held via AltLeft/AltRight (laptop stand-in for `&mo 1` / `&mo 2`). */
	#altHold: string | null = null;
	/** Layer inferred from HID (arrows, F-keys, ZMK symbol keycodes). */
	#inferredHold: string | null = null;
	#inferredCodes = new Set<string>();

	activeLayer = $derived.by(() => {
		const id = this.holdLayerId ?? this.activeLayerId;
		return layerById(this.board, id);
	});

	focusIndex = $derived(this.selectedKeyIndex ?? this.hoveredKeyIndex);

	selectedBinding = $derived.by((): Binding | null => {
		if (this.focusIndex === null) return null;
		return bindingAt(this.activeLayer, this.focusIndex);
	});

	baseLayer = $derived(this.board.layers[0] as Layer);

	applyLayers(layers: Layer[], origin: ConfigOrigin, warnings: string[] = []) {
		this.board = {
			...this.board,
			layers,
			sources:
				origin.kind === 'github'
					? [
							{ title: origin.label, url: origin.url },
							...lily58.sources.filter((s) => s.url !== origin.url)
						]
					: this.board.sources
		};
		this.configOrigin = origin;
		this.configWarnings = warnings;
		this.#hitMap = buildZmkHitMap(layers);
		this.#heldCodes.clear();
		this.#codeToIndex.clear();
		this.#altHold = null;
		this.#inferredHold = null;
		this.#inferredCodes.clear();
		this.pressedIndices = [];
		this.lastPhysical = null;
		this.selectedKeyIndex = null;
		this.holdLayerId = null;
		this.activeLayerId = layers[0]?.id ?? 'base';
	}

	resetToBuiltin() {
		this.board = structuredClone(lily58);
		this.applyLayers(this.board.layers, { kind: 'builtin', label: 'PandaKB stock' });
	}

	setLayer(id: string) {
		this.activeLayerId = id;
		this.holdLayerId = null;
		this.#altHold = null;
		this.#inferredHold = null;
		this.#inferredCodes.clear();
	}

	selectKey(index: number | null) {
		this.selectedKeyIndex = index;
	}

	hoverKey(index: number | null) {
		this.hoveredKeyIndex = index;
	}

	holdMomentary(binding: Binding | null) {
		if (isMomentaryLayer(binding) && binding?.params[0]) {
			const target = this.board.layers[Number(binding.params[0])];
			this.holdLayerId = target?.id ?? null;
			return;
		}
		this.holdLayerId = null;
	}

	/** Firmware LAYER_PREVIEW (0x20) — show overlay layer while held. */
	setFirmwarePreview(layerIndex: number | null) {
		if (layerIndex == null) {
			this.holdLayerId = this.#altHold ?? this.#inferredHold;
			const moIndices = this.baseLayer.bindings
				.map((b, i) => (isMomentaryLayer(b) ? i : -1))
				.filter((i) => i >= 0);
			this.pressedIndices = this.pressedIndices.filter((i) => !moIndices.includes(i));
			return;
		}
		const target = this.board.layers[layerIndex];
		this.holdLayerId = target?.id ?? null;
		const mo = this.baseLayer.bindings.findIndex(
			(b) => isMomentaryLayer(b) && b.params[0] === String(layerIndex)
		);
		if (mo >= 0 && !this.pressedIndices.includes(mo)) {
			this.pressedIndices = [...this.pressedIndices, mo];
		}
	}

	releaseHold() {
		this.holdLayerId = this.#altHold ?? this.#inferredHold;
	}

	isPressed(index: number): boolean {
		return this.pressedIndices.includes(index);
	}

	#syncHoldLayer() {
		this.holdLayerId = this.#altHold ?? this.#inferredHold;
	}

	#preferredLayerId(): string | null {
		return this.#altHold ?? this.#inferredHold ?? (this.activeLayerId !== 'base' ? this.activeLayerId : null);
	}

	handlePhysicalKeyDown(event: KeyboardEvent) {
		if (event.repeat) return;
		const target = event.target as HTMLElement | null;
		if (
			target &&
			(target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
		) {
			return;
		}

		// Laptop stand-in: AltLeft / AltRight preview `&mo 1` / `&mo 2` targets while held.
		// Real Lily58 `&mo` keys never reach the host.
		if (event.code === 'AltLeft') {
			event.preventDefault();
			this.#heldCodes.add(event.code);
			const target = this.board.layers[1];
			this.#altHold = target?.id ?? 'lower';
			this.#syncHoldLayer();
			const mo = this.baseLayer.bindings.findIndex(
				(b) => isMomentaryLayer(b) && b.params[0] === '1'
			);
			if (mo >= 0 && !this.pressedIndices.includes(mo)) {
				this.pressedIndices = [...this.pressedIndices, mo];
			}
			this.lastPhysical = {
				index: mo >= 0 ? mo : 0,
				layerId: target?.id ?? 'lower',
				code: 'MO_1',
				impliesLayer: true,
				shiftHeld: event.shiftKey
			};
			return;
		}
		if (event.code === 'AltRight') {
			event.preventDefault();
			this.#heldCodes.add(event.code);
			const target = this.board.layers[2];
			this.#altHold = target?.id ?? null;
			this.#syncHoldLayer();
			const mo = this.baseLayer.bindings.findIndex(
				(b) => isMomentaryLayer(b) && b.params[0] === '2'
			);
			if (mo >= 0 && !this.pressedIndices.includes(mo)) {
				this.pressedIndices = [...this.pressedIndices, mo];
			}
			this.lastPhysical = {
				index: mo >= 0 ? mo : 0,
				layerId: target?.id ?? 'adjust',
				code: 'MO_2',
				impliesLayer: true,
				shiftHeld: event.shiftKey
			};
			return;
		}

		this.#heldCodes.add(event.code);

		const physicalShiftHeld =
			this.#heldCodes.has('ShiftLeft') || this.#heldCodes.has('ShiftRight');

		const resolved = resolvePhysicalPress(event, this.#hitMap, {
			preferredLayerId: this.#preferredLayerId(),
			activeLayerId: this.activeLayerId,
			physicalShiftHeld
		});

		if (!resolved) return;

		event.preventDefault();

		if (resolved.impliesLayer && resolved.layerId !== 'base') {
			this.#inferredHold = resolved.layerId;
			this.#inferredCodes.add(event.code);
			this.#syncHoldLayer();
		}

		if (!this.pressedIndices.includes(resolved.index)) {
			this.pressedIndices = [...this.pressedIndices, resolved.index];
		}
		this.#codeToIndex.set(event.code, resolved.index);

		this.selectedKeyIndex = resolved.index;
		this.lastPhysical = resolved;
	}

	handlePhysicalKeyUp(event: KeyboardEvent) {
		if (event.code === 'AltLeft' || event.code === 'AltRight') {
			this.#heldCodes.delete(event.code);
			if (!this.#heldCodes.has('AltLeft') && !this.#heldCodes.has('AltRight')) {
				this.#altHold = null;
				const moIndices = this.baseLayer.bindings
					.map((b, i) => (isMomentaryLayer(b) ? i : -1))
					.filter((i) => i >= 0);
				this.pressedIndices = this.pressedIndices.filter((i) => !moIndices.includes(i));
				this.#syncHoldLayer();
			}
			return;
		}

		this.#heldCodes.delete(event.code);

		if (this.#inferredCodes.has(event.code)) {
			this.#inferredCodes.delete(event.code);
			if (this.#inferredCodes.size === 0) {
				this.#inferredHold = null;
				this.#syncHoldLayer();
			}
		}

		const index = this.#codeToIndex.get(event.code);
		this.#codeToIndex.delete(event.code);
		if (index !== undefined) {
			this.pressedIndices = this.pressedIndices.filter((i) => i !== index);
			const binding = bindingAt(this.baseLayer, index);
			if (isMomentaryLayer(binding)) {
				this.releaseHold();
			}
		}
	}
}

export const boardState = new BoardState();
