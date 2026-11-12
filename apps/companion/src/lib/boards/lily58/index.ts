import type { BoardDefinition, PhysicalKey } from '#lib/keymap/types';
import layoutRaw from './layout.json';
import keymapRaw from './keymap.json';

type LayoutEntry = {
	row: number;
	col: number;
	x: number;
	y: number;
	w?: number;
	h?: number;
	r?: number;
	rx?: number;
	ry?: number;
};

const layoutEntries = (layoutRaw.layouts.default_transform.layout ?? []) as LayoutEntry[];

const keys: PhysicalKey[] = layoutEntries.map((entry, index) => ({
	index,
	row: entry.row,
	col: entry.col,
	x: entry.x,
	y: entry.y,
	w: entry.w ?? 1,
	h: entry.h ?? 1,
	r: entry.r ?? 0,
	rx: entry.rx ?? entry.x,
	ry: entry.ry ?? entry.y
}));

export const lily58: BoardDefinition = {
	id: 'pandakb-lily58',
	name: 'Lily58',
	vendor: 'PandaKB',
	keys,
	layers: keymapRaw.layers,
	display: {
		kind: 'oled',
		width: 128,
		height: 64
	},
	sources: [
		{
			title: 'PandaKB ZMK (lily branch)',
			url: 'https://github.com/PandaKBLab/zmk-for-keyboards/tree/zmk-for-lily'
		},
		{
			title: 'Typeractive Lily58 Wireless firmware',
			url: 'https://docs.typeractive.xyz/build-guides/lily58-wireless/firmware'
		},
		{
			title: 'Keymap Editor',
			url: 'https://github.com/nickcoutsos/keymap-editor'
		},
		{
			title: 'ZMK Studio',
			url: 'https://zmk.studio/'
		}
	]
};
