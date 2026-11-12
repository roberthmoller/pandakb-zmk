<script lang="ts">
	import { bindingAt } from '#lib/keymap/resolve';
	import { boardState } from '#lib/state/board.svelte';
	import { typingState } from '#lib/state/typing.svelte';

	function labelForIndex(index: number): string {
		const overlay = typingState.overlayForIndex(index);
		if (overlay) return overlay.label;
		const binding = bindingAt(boardState.baseLayer, index);
		return binding?.label || `#${index}`;
	}
</script>

<section
	class="flex flex-wrap items-end justify-between gap-6 border-t border-white/10 pt-4"
	aria-label="Trainer stats"
>
	{#if typingState.isGameLayout}
		<div class="flex flex-wrap gap-8">
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Score
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.score}</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Accuracy
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.accuracy}%</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Streak
				</div>
				<div class="text-3xl font-semibold tracking-tight">
					{typingState.streak}
					<span class="text-lg text-muted-foreground">×{typingState.multiplier}</span>
				</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Misses
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.misses}</div>
			</div>
		</div>
	{:else}
		<div class="flex flex-wrap gap-8">
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					WPM
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.wpm}</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Accuracy
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.accuracy}%</div>
			</div>
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Errors
				</div>
				<div class="text-3xl font-semibold tracking-tight">{typingState.errors}</div>
			</div>
		</div>
	{/if}

	{#if typingState.topMisses.length > 0}
		<div class="flex flex-wrap items-center gap-2">
			<span class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground"
				>Miss hotspots</span
			>
			{#each typingState.topMisses as miss (miss.index)}
				<span class="font-mono text-sm text-muted-foreground">
					{labelForIndex(miss.index)}
					<span class="text-foreground">{miss.count}×</span>
				</span>
			{/each}
		</div>
	{/if}
</section>
