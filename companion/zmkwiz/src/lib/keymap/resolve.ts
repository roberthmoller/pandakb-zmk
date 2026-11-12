import type { Binding, BoardDefinition, Layer } from './types';

export function layerById(board: BoardDefinition, layerId: string): Layer {
	return board.layers.find((layer) => layer.id === layerId) ?? board.layers[0];
}

export function bindingAt(layer: Layer, index: number): Binding | null {
	return layer.bindings[index] ?? null;
}

export function compareLayers(
	board: BoardDefinition,
	index: number
): { layer: Layer; binding: Binding | null }[] {
	return board.layers.map((layer) => ({
		layer,
		binding: bindingAt(layer, index)
	}));
}
