<script lang="ts">
	import LayerSwitcher from '#lib/components/LayerSwitcher.svelte';
	import { boardState } from '#lib/state/board.svelte';

	const focus = $derived(boardState.focusIndex);
	const key = $derived(focus === null ? null : boardState.board.keys[focus]);
	const binding = $derived(boardState.selectedBinding);
	const holding = $derived(boardState.holdLayerId);
	const last = $derived(boardState.lastPhysical);
	const shiftLit = $derived(
		boardState.pressedIndices.some((index) => {
			const b = boardState.baseLayer.bindings[index];
			return b?.behavior === 'kp' && (b.params[0] === 'LSHFT' || b.params[0] === 'RSHFT');
		})
	);
</script>

<section class="space-y-3" aria-label="Learn">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				Inspect key
			</div>
			{#if holding}
				<p class="mt-1 text-sm text-muted-foreground">
					Showing <strong class="text-foreground">{boardState.activeLayer.name}</strong>
					{#if last?.impliesLayer}
						· detected from your keypress
					{:else}
						while held
					{/if}
				</p>
			{/if}
		</div>
		<LayerSwitcher />
	</div>

	<div class="rounded-lg bg-[#171717] px-5 py-6 md:px-6 md:py-8" aria-live="polite">
		{#if focus === null || !key}
			<div class="font-mono text-2xl tracking-tight text-muted-foreground md:text-3xl">
				Press a key
			</div>
			<p class="mt-3 max-w-2xl text-sm text-muted-foreground">
				Type on your Lily58 or laptop. Lower / Raise are detected from layer keys (symbols, F-keys,
				arrows). On a laptop, hold Left Alt / Right Alt to preview those layers.
			</p>
		{:else}
			<div class="font-mono text-3xl tracking-tight md:text-4xl">
				{#if shiftLit && binding?.label && binding.label.length === 1 && binding.label === binding.label.toLowerCase()}
					{binding.label.toUpperCase()}
				{:else}
					{binding?.label || 'Empty'}
				{/if}
			</div>
			<p class="mt-2 font-mono text-base text-muted-foreground md:text-lg">
				{binding?.raw ?? '—'}
				{#if shiftLit}
					<span class="text-foreground"> + Shift</span>
				{/if}
			</p>
			{#if binding?.gloss}
				<p class="mt-4 max-w-2xl text-sm text-muted-foreground">{binding.gloss}</p>
			{/if}
		{/if}
	</div>

	<div
		class="flex min-h-14 flex-wrap items-center gap-4 rounded-lg px-4 py-3 {focus !== null
			? 'bg-[#1c1c1c]'
			: 'bg-transparent'}"
		aria-live="polite"
	>
		{#if focus !== null && key}
			<span class="font-mono text-[0.65rem] uppercase tracking-widest text-foreground">Selected</span>
			<span class="font-mono text-xl">
				#{key.index}
				<span class="text-muted-foreground">row {key.row}, col {key.col}</span>
			</span>
			<span class="font-mono text-xl text-muted-foreground">
				layer <strong class="text-foreground">{boardState.activeLayer.name}</strong>
			</span>
			{#if shiftLit}
				<span class="font-mono text-sm text-foreground">Shift held</span>
			{/if}
		{:else}
			<span class="font-mono text-sm text-muted-foreground">
				Waiting for a keypress — the board lights up as you type.
			</span>
		{/if}
	</div>
</section>
