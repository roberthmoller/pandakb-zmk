import type { Binding } from '#lib/keymap/types';

/** Momentary layer behaviors: stock `&mo` and companion `&mo_prev`. */
export function isMomentaryLayer(binding: Binding | null | undefined): boolean {
	const b = binding?.behavior;
	return b === 'mo' || b === 'mo_prev';
}
