import type { GetBehaviorDetailsResponse } from '@zmkfirmware/zmk-studio-ts-client/behaviors';
import type { BehaviorBinding, Keymap, Layer } from '@zmkfirmware/zmk-studio-ts-client/keymap';
import {
	AddLayerErrorCode,
	RemoveLayerErrorCode,
	SetLayerBindingResponse,
	SetLayerPropsResponse
} from '@zmkfirmware/zmk-studio-ts-client/keymap';
import { studioConnection } from './connection.svelte';
import { hidUsageLabel } from './hid';

class LiveKeymap {
	keymap = $state<Keymap | null>(null);
	behaviors = $state<Record<number, GetBehaviorDetailsResponse>>({});
	activeLayerIndex = $state(0);
	selectedKey = $state<number | null>(null);
	unsaved = $state(false);
	busy = $state(false);
	message = $state<string | null>(null);

	activeLayer = $derived.by((): Layer | null => {
		const layers = this.keymap?.layers;
		if (!layers?.length) return null;
		return layers[this.activeLayerIndex] ?? layers[0] ?? null;
	});

	selectedBinding = $derived.by((): BehaviorBinding | null => {
		if (this.selectedKey === null || !this.activeLayer) return null;
		return this.activeLayer.bindings[this.selectedKey] ?? null;
	});

	behaviorList = $derived(
		Object.values(this.behaviors).sort((a, b) => a.displayName.localeCompare(b.displayName))
	);

	canAddLayer = $derived((this.keymap?.availableLayers ?? 0) > 0);
	canRemoveLayer = $derived((this.keymap?.layers.length ?? 0) > 1);

	labelFor(binding: BehaviorBinding | null | undefined): string {
		if (!binding) return '';
		const details = this.behaviors[binding.behaviorId];
		const name = details?.displayName ?? `#${binding.behaviorId}`;
		if (!binding.param1 && !binding.param2) return name;
		const p1 = binding.param1 ? hidUsageLabel(binding.param1) : '';
		// Layer ids are small ints; HID usages are large. Prefer HID label when page bits set.
		const p1Text =
			binding.param1 > 0xffff ? p1 : binding.param1 ? String(binding.param1) : '';
		if (name.toLowerCase().includes('key press') || name === 'Key Press') {
			return p1 || name;
		}
		if (
			name.toLowerCase().includes('momentary') ||
			name.toLowerCase().includes('to layer') ||
			name.toLowerCase().includes('toggle layer') ||
			name.toLowerCase().startsWith('layer')
		) {
			const target = this.keymap?.layers.find((l) => l.id === binding.param1);
			const label = target ? this.layerLabel(target) : String(binding.param1);
			if (name.toLowerCase().includes('momentary') || name.startsWith('Momentary')) {
				return `mo ${label}`;
			}
			return `${name} ${label}`;
		}
		return p1Text ? `${name} ${p1Text}` : name;
	}

	async refresh() {
		if (!studioConnection.conn) {
			this.keymap = null;
			this.behaviors = {};
			return;
		}
		this.busy = true;
		this.message = null;
		try {
			const [kmRes, unsavedRes, behaviorIdsRes] = await Promise.all([
				studioConnection.rpc({ keymap: { getKeymap: true } }),
				studioConnection.rpc({ keymap: { checkUnsavedChanges: true } }),
				studioConnection.rpc({ behaviors: { listAllBehaviors: true } })
			]);

			this.keymap = kmRes.keymap?.getKeymap ?? null;
			this.unsaved = Boolean(unsavedRes.keymap?.checkUnsavedChanges);

			const ids = behaviorIdsRes.behaviors?.listAllBehaviors?.behaviors ?? [];
			const map: Record<number, GetBehaviorDetailsResponse> = {};
			for (const behaviorId of ids) {
				const details = await studioConnection.rpc({
					behaviors: { getBehaviorDetails: { behaviorId } }
				});
				const dets = details.behaviors?.getBehaviorDetails;
				if (dets) map[dets.id] = dets;
			}
			this.behaviors = map;

			if (this.activeLayerIndex >= (this.keymap?.layers.length ?? 0)) {
				this.activeLayerIndex = 0;
			}
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	selectLayer(index: number) {
		this.activeLayerIndex = index;
		this.selectedKey = null;
	}

	selectKey(index: number) {
		this.selectedKey = index;
	}

	layerLabel(layer: Layer | null | undefined, index?: number): string {
		if (!layer) return 'Layer';
		if (layer.name?.trim()) return layer.name;
		const i = index ?? this.keymap?.layers.findIndex((l) => l.id === layer.id) ?? -1;
		return i >= 0 ? `Layer ${i}` : `Layer #${layer.id}`;
	}

	async addLayer() {
		if (!studioConnection.conn || !this.canAddLayer) return;
		this.busy = true;
		this.message = null;
		try {
			const res = await studioConnection.rpc({ keymap: { addLayer: {} } });
			const add = res.keymap?.addLayer;
			if (add?.err !== undefined && add.err !== AddLayerErrorCode.ADD_LAYER_ERR_OK) {
				const reason =
					add.err === AddLayerErrorCode.ADD_LAYER_ERR_NO_SPACE
						? 'no free layer slots (rebuild with more CONFIG_ZMK_KEYMAP_LAYERS)'
						: `error ${add.err}`;
				throw new Error(`addLayer failed (${reason})`);
			}
			const ok = add?.ok;
			if (!ok?.layer || !this.keymap) {
				await this.refresh();
				return;
			}
			const layers = [...this.keymap.layers];
			const insertAt = Math.min(ok.index, layers.length);
			layers.splice(insertAt, 0, ok.layer);
			this.keymap = {
				...this.keymap,
				layers,
				availableLayers: Math.max(0, this.keymap.availableLayers - 1)
			};
			this.activeLayerIndex = insertAt;
			this.selectedKey = null;
			this.unsaved = true;
			this.message = `Added ${this.layerLabel(ok.layer, insertAt)}. Map keys, then Save.`;
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async removeLayer(index = this.activeLayerIndex) {
		if (!studioConnection.conn || !this.canRemoveLayer || !this.keymap) return;
		if (index < 0 || index >= this.keymap.layers.length) return;
		this.busy = true;
		this.message = null;
		try {
			const removed = this.keymap.layers[index];
			const res = await studioConnection.rpc({
				keymap: { removeLayer: { layerIndex: index } }
			});
			const rm = res.keymap?.removeLayer;
			if (rm?.err !== undefined && rm.err !== RemoveLayerErrorCode.REMOVE_LAYER_ERR_OK) {
				throw new Error(`removeLayer failed (${rm.err})`);
			}
			const layers = this.keymap.layers.filter((_, i) => i !== index);
			this.keymap = {
				...this.keymap,
				layers,
				availableLayers: this.keymap.availableLayers + 1
			};
			this.activeLayerIndex = Math.min(this.activeLayerIndex, layers.length - 1);
			if (this.activeLayerIndex < 0) this.activeLayerIndex = 0;
			this.selectedKey = null;
			this.unsaved = true;
			this.message = `Removed ${this.layerLabel(removed, index)}. Save to keep the change.`;
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async renameLayer(name: string, layerId = this.activeLayer?.id) {
		if (!studioConnection.conn || layerId === undefined || !this.keymap) return;
		const max = this.keymap.maxLayerNameLength || 16;
		const next = name.trim().slice(0, max);
		if (!next) {
			this.message = 'Layer name cannot be empty.';
			return;
		}
		this.busy = true;
		this.message = null;
		try {
			const res = await studioConnection.rpc({
				keymap: { setLayerProps: { layerId, name: next } }
			});
			const code = res.keymap?.setLayerProps;
			if (
				code !== undefined &&
				code !== SetLayerPropsResponse.SET_LAYER_PROPS_RESP_OK
			) {
				throw new Error(`setLayerProps failed (${code})`);
			}
			this.keymap = {
				...this.keymap,
				layers: this.keymap.layers.map((l) => (l.id === layerId ? { ...l, name: next } : l))
			};
			this.unsaved = true;
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async setBinding(keyPosition: number, binding: BehaviorBinding) {
		const layer = this.activeLayer;
		if (!layer || !studioConnection.conn) return;
		this.busy = true;
		this.message = null;
		try {
			const res = await studioConnection.rpc({
				keymap: {
					setLayerBinding: {
						layerId: layer.id,
						keyPosition,
						binding
					}
				}
			});
			const code = res.keymap?.setLayerBinding;
			if (code !== undefined && code !== SetLayerBindingResponse.SET_LAYER_BINDING_RESP_OK) {
				throw new Error(`setLayerBinding failed (${code})`);
			}

			if (this.keymap) {
				const layers = this.keymap.layers.map((l, i) => {
					if (i !== this.activeLayerIndex) return l;
					const bindings = l.bindings.slice();
					bindings[keyPosition] = binding;
					return { ...l, bindings };
				});
				this.keymap = { ...this.keymap, layers };
			}
			this.unsaved = true;
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async save() {
		if (!studioConnection.conn) return;
		this.busy = true;
		this.message = null;
		try {
			const res = await studioConnection.rpc({ keymap: { saveChanges: true } });
			const err = res.keymap?.saveChanges?.err;
			if (err) throw new Error(`Save failed (${err})`);
			this.unsaved = false;
			this.message = 'Saved to keyboard.';
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async discard() {
		if (!studioConnection.conn) return;
		this.busy = true;
		this.message = null;
		try {
			await studioConnection.rpc({ keymap: { discardChanges: true } });
			await this.refresh();
			this.message = 'Discarded unsaved changes.';
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	async resetStock() {
		if (!studioConnection.conn) return;
		this.busy = true;
		this.message = null;
		try {
			await studioConnection.rpc({ core: { resetSettings: true } });
			await this.refresh();
			this.message = 'Restored stock keymap from firmware.';
		} catch (err) {
			this.message = err instanceof Error ? err.message : String(err);
		} finally {
			this.busy = false;
		}
	}

	clear() {
		this.keymap = null;
		this.behaviors = {};
		this.activeLayerIndex = 0;
		this.selectedKey = null;
		this.unsaved = false;
		this.message = null;
	}
}

export const liveKeymap = new LiveKeymap();
