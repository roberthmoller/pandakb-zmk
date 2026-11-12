<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { boardState } from '#lib/state/board.svelte';

	const layers = $derived(boardState.board.layers);
	const activeId = $derived(boardState.holdLayerId ?? boardState.activeLayerId);
</script>

<div class="surface-panel inline-flex max-w-full" role="tablist" aria-label="Layers">
	<div class="surface-panel-inner flex flex-wrap gap-1 p-1">
		{#each layers as layer (layer.id)}
			<Button
				type="button"
				size="sm"
				variant={activeId === layer.id ? 'default' : 'ghost'}
				role="tab"
				aria-selected={activeId === layer.id}
				onclick={() => boardState.setLayer(layer.id)}
			>
				{layer.name}
			</Button>
		{/each}
	</div>
</div>
