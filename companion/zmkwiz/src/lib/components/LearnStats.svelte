<script lang="ts">
	import { compareLayers } from '#lib/keymap/resolve';
	import { boardState } from '#lib/state/board.svelte';

	const focus = $derived(boardState.focusIndex);
	const comparisons = $derived(focus === null ? [] : compareLayers(boardState.board, focus));
	const layerName = $derived(boardState.activeLayer.name);
</script>

<section
	class="flex flex-wrap items-end justify-between gap-6 border-t border-white/10 pt-4"
	aria-label="Learn details"
>
	{#if comparisons.length > 0}
		<div class="flex min-w-0 flex-1 flex-wrap gap-x-6 gap-y-2">
			<span class="w-full font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground"
				>All layers</span
			>
			{#each comparisons as item (item.layer.id)}
				<div
					class="min-w-28 {item.layer.id === boardState.activeLayer.id
						? 'text-foreground'
						: 'text-muted-foreground'}"
				>
					<div class="font-mono text-[0.65rem] uppercase tracking-widest">{item.layer.name}</div>
					<div class="font-mono text-xl font-semibold tracking-tight">
						{item.binding?.label || '—'}
					</div>
					<div class="truncate font-mono text-xs opacity-70">{item.binding?.raw ?? '—'}</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="flex flex-wrap gap-8">
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Layer
				</div>
				<div class="text-3xl font-semibold tracking-tight">{layerName}</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Keys
				</div>
				<div class="text-3xl font-semibold tracking-tight">{boardState.board.keys.length}</div>
			</div>
		</div>
	{/if}

	<div
		class="grid h-16 w-40 shrink-0 grid-rows-[auto_1fr_auto] gap-0.5 rounded bg-[#0a0a0a] px-2.5 py-1.5 font-mono text-[0.65rem] text-[#e8e8e8]"
		aria-label="OLED preview"
	>
		<div class="flex justify-between gap-2">
			<span class="font-semibold uppercase">{layerName}</span>
			<span>87%</span>
		</div>
		<div class="flex items-end justify-between opacity-80">
			<span>▌▌▌▌▖</span>
			<span>BT1</span>
		</div>
		<div class="flex justify-between opacity-70">
			<span>Lily58</span>
			<span>BLE</span>
		</div>
	</div>
</section>
