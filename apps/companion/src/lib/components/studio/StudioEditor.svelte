<script lang="ts">
	import BehaviorPicker from '#lib/components/studio/BehaviorPicker.svelte';
	import ConnectBar from '#lib/components/studio/ConnectBar.svelte';
	import KeyCap from '#lib/components/KeyCap.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { boardState } from '#lib/state/board.svelte';
	import { studioConnection } from '#lib/studio/connection.svelte';
	import { liveKeymap } from '#lib/studio/liveKeymap.svelte';
	import type { Binding } from '#lib/keymap/types';
	import { onMount } from 'svelte';

	const pad = 16;
	const maxUnit = 56;
	const minUnit = 26;

	let vizEl: HTMLDivElement | undefined = $state();
	let unit = $state(40);

	const layoutSize = $derived.by(() => {
		const keys = boardState.board.keys;
		let minX = 0;
		let minY = 0;
		let maxX = 1;
		let maxY = 1;
		for (const key of keys) {
			minX = Math.min(minX, key.x);
			minY = Math.min(minY, key.y);
			maxX = Math.max(maxX, key.x + key.w);
			maxY = Math.max(maxY, key.y + key.h);
		}
		return { minX, minY, width: maxX - minX, height: maxY - minY };
	});

	const bounds = $derived({
		width: layoutSize.width * unit + pad * 2,
		height: layoutSize.height * unit + pad * 2
	});

	const originOffset = $derived({
		x: pad - layoutSize.minX * unit,
		y: pad - layoutSize.minY * unit
	});

	function fitUnit(width: number) {
		if (width <= 0 || layoutSize.width <= 0) return;
		unit = Math.max(minUnit, Math.min(maxUnit, Math.floor((width - pad * 2) / layoutSize.width)));
	}

	onMount(() => {
		if (!vizEl) return;
		const observer = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (!entry) return;
			fitUnit(entry.contentRect.width);
		});
		observer.observe(vizEl);
		fitUnit(vizEl.clientWidth);
		return () => observer.disconnect();
	});

	let renameDraft = $state('');

	$effect(() => {
		const layer = liveKeymap.activeLayer;
		renameDraft = layer?.name?.trim() || liveKeymap.layerLabel(layer, liveKeymap.activeLayerIndex);
	});

	function fakeBinding(index: number): Binding {
		const layer = liveKeymap.activeLayer;
		const b = layer?.bindings[index];
		const label = liveKeymap.labelFor(b);
		return {
			raw: label,
			behavior: 'kp',
			params: [],
			label,
			gloss: ''
		};
	}

	function commitRename() {
		const layer = liveKeymap.activeLayer;
		if (!layer) return;
		const current = layer.name?.trim() || '';
		const next = renameDraft.trim();
		if (!next || next === current) return;
		liveKeymap.renameLayer(next, layer.id);
	}
</script>

<section class="space-y-4" aria-label="Live Studio editor">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				Live remap
			</div>
			<p class="mt-1 max-w-2xl text-sm text-muted-foreground">
				Studio RPC over Web Serial — add or remove layers, map keys, and save to the keyboard
				without rebuilding (until you run out of free layer slots).
			</p>
		</div>
		{#if studioConnection.status === 'connected'}
			<div class="flex flex-wrap gap-2">
				<Button
					size="sm"
					onclick={() => liveKeymap.save()}
					disabled={liveKeymap.busy || !liveKeymap.unsaved}
				>
					Save to keyboard
				</Button>
				<Button
					size="sm"
					variant="outline"
					onclick={() => liveKeymap.discard()}
					disabled={liveKeymap.busy || !liveKeymap.unsaved}
				>
					Discard
				</Button>
				<Button
					size="sm"
					variant="ghost"
					onclick={() => liveKeymap.resetStock()}
					disabled={liveKeymap.busy}
				>
					Restore stock
				</Button>
			</div>
		{/if}
	</div>

	<ConnectBar />

	{#if liveKeymap.message}
		<p class="font-mono text-sm text-muted-foreground">{liveKeymap.message}</p>
	{/if}

	{#if studioConnection.status === 'connected' && liveKeymap.keymap}
		{#if !studioConnection.unlocked}
			<p class="rounded-lg border border-white/10 px-4 py-3 text-sm text-muted-foreground">
				Device reports <strong class="text-foreground">locked</strong>. Your firmware has locking
				disabled — refresh; if it stays locked, add
				<code class="text-foreground">&amp;studio_unlock</code> or rebuild with
				<code class="text-foreground">CONFIG_ZMK_STUDIO_LOCKING=n</code>.
			</p>
		{/if}

		<div class="flex flex-wrap items-center gap-2">
			<div class="flex flex-wrap gap-1" role="tablist" aria-label="Device layers">
				{#each liveKeymap.keymap.layers as layer, i (layer.id)}
					<Button
						type="button"
						size="sm"
						variant={liveKeymap.activeLayerIndex === i ? 'default' : 'ghost'}
						onclick={() => liveKeymap.selectLayer(i)}
					>
						{liveKeymap.layerLabel(layer, i)}
					</Button>
				{/each}
			</div>
			<div class="flex flex-wrap gap-1">
				<Button
					type="button"
					size="sm"
					variant="outline"
					onclick={() => liveKeymap.addLayer()}
					disabled={liveKeymap.busy || !liveKeymap.canAddLayer}
					title={liveKeymap.canAddLayer
						? `${liveKeymap.keymap.availableLayers} free slot(s)`
						: 'No free layer slots — rebuild firmware with more layers'}
				>
					Add layer
				</Button>
				<Button
					type="button"
					size="sm"
					variant="ghost"
					onclick={() => liveKeymap.removeLayer()}
					disabled={liveKeymap.busy || !liveKeymap.canRemoveLayer}
					title={liveKeymap.canRemoveLayer
						? 'Remove the selected layer'
						: 'At least one layer must remain'}
				>
					Remove
				</Button>
			</div>
			{#if liveKeymap.keymap.availableLayers > 0}
				<span class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					{liveKeymap.keymap.availableLayers} free
				</span>
			{/if}
		</div>

		<div class="flex max-w-sm flex-wrap items-end gap-2">
			<label class="min-w-[12rem] flex-1 space-y-1 text-sm">
				<span class="text-muted-foreground">Layer name</span>
				<input
					class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
					maxlength={liveKeymap.keymap.maxLayerNameLength || 16}
					bind:value={renameDraft}
					disabled={liveKeymap.busy || !liveKeymap.activeLayer}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							commitRename();
						}
					}}
				/>
			</label>
			<Button
				type="button"
				size="sm"
				variant="outline"
				onclick={commitRename}
				disabled={liveKeymap.busy || !liveKeymap.activeLayer}
			>
				Rename
			</Button>
		</div>

		<div class="grid gap-6 lg:grid-cols-[1fr_20rem]">
			<div class="min-w-0" bind:this={vizEl}>
				<div class="relative mx-auto" style:width="{bounds.width}px" style:height="{bounds.height}px">
					<div
						class="absolute inset-0"
						style:transform="translate({originOffset.x}px, {originOffset.y}px)"
					>
						{#each boardState.board.keys as keyDef (keyDef.index)}
							{@const binding = fakeBinding(keyDef.index)}
							<KeyCap
								{keyDef}
								{binding}
								{unit}
								order={keyDef.index}
								selected={liveKeymap.selectedKey === keyDef.index}
								hovered={false}
								layerAccent={false}
								pressed={false}
								interactive={true}
								onclick={() => liveKeymap.selectKey(keyDef.index)}
								onhover={() => {}}
								onleave={() => {}}
								onpress={() => {}}
								onrelease={() => {}}
							/>
						{/each}
					</div>
				</div>
			</div>

			<div>
				{#if liveKeymap.selectedKey !== null && liveKeymap.selectedBinding}
					<p class="mb-2 font-mono text-xs text-muted-foreground">
						Key #{liveKeymap.selectedKey} · {liveKeymap.layerLabel(
							liveKeymap.activeLayer,
							liveKeymap.activeLayerIndex
						)}
					</p>
					<BehaviorPicker
						binding={liveKeymap.selectedBinding}
						onapply={(next) => {
							if (liveKeymap.selectedKey !== null) {
								liveKeymap.setBinding(liveKeymap.selectedKey, next);
							}
						}}
					/>
				{:else}
					<p class="text-sm text-muted-foreground">
						Select a key on {liveKeymap.layerLabel(
							liveKeymap.activeLayer,
							liveKeymap.activeLayerIndex
						)} to edit its mapping.
					</p>
				{/if}
			</div>
		</div>
	{:else if studioConnection.status === 'connected' && liveKeymap.busy}
		<p class="text-sm text-muted-foreground">Loading keymap…</p>
	{/if}
</section>
