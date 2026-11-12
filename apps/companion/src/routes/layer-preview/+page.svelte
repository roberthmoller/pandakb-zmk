<script lang="ts">
	import KeyboardViz from '#lib/components/KeyboardViz.svelte';
	import { loadFirmwareKeymap } from '#lib/firmware/loadKeymap';
	import { isTauri } from '#lib/firmware/tauri';
	import { boardState } from '#lib/state/board.svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	type PreviewEvent = { show: boolean; layer: number };
	type KeyMatrixEvent = { pressed: number[] };

	let visible = $state(false);
	let layerName = $state('Layer');
	let ready = $state(false);
	let hint = $state('Release the layer key to dismiss');
	/** Last KEY_MATRIX — restored after keymap reload (applyLayers clears presses). */
	let lastMatrix: number[] = [];

	function hintForLayer(id: string | undefined): string {
		switch (id) {
			case 'adjust':
				return 'Release Lower or Raise to dismiss';
			case 'wow':
			case 'lol':
				return 'Switch to Base to dismiss';
			default:
				return 'Release the layer key to dismiss';
		}
	}

	onMount(() => {
		const unlisteners: Array<() => void> = [];

		void (async () => {
			if (isTauri()) {
				await loadFirmwareKeymap();
				ready = true;
			} else {
				ready = true;
			}

			try {
				unlisteners.push(
					await listen<PreviewEvent>('companion://layer-preview', (event) => {
						const { show, layer } = event.payload;
						void (async () => {
							if (show) {
								// Re-read local firmware so overlay matches latest edits.
								if (isTauri()) await loadFirmwareKeymap();
								boardState.setFirmwarePreview(layer);
								boardState.setPressedFromMatrix(lastMatrix);
								layerName = boardState.activeLayer?.name ?? `Layer ${layer}`;
								hint = hintForLayer(boardState.activeLayer?.id);
								visible = true;
							} else {
								lastMatrix = [];
								boardState.setFirmwarePreview(null);
								visible = false;
							}
						})();
					})
				);

				unlisteners.push(
					await listen<KeyMatrixEvent>('companion://key-matrix', (event) => {
						lastMatrix = event.payload.pressed ?? [];
						boardState.setPressedFromMatrix(lastMatrix);
					})
				);
			} catch {
				// Browser / non-Tauri preview — stay idle.
			}
		})();

		return () => {
			for (const off of unlisteners) off();
		};
	});
</script>

<div class="overlay-root" class:visible aria-hidden={!visible}>
	{#if ready}
		<header class="header">
			<span class="eyebrow">Layer preview</span>
			<strong class="title">{layerName}</strong>
			<span class="hint">{hint}</span>
		</header>
		<div class="board-wrap">
			<KeyboardViz />
		</div>
	{/if}
</div>

<style>
	:global(html),
	:global(body) {
		/* Native HudWindow vibrancy sits behind the webview — stay clear. */
		background: transparent !important;
		overflow: hidden;
	}

	.overlay-root {
		box-sizing: border-box;
		display: flex;
		min-height: 100vh;
		flex-direction: column;
		align-items: stretch;
		justify-content: center;
		gap: 0.75rem;
		padding: 1.25rem 1.5rem 1.5rem;
		opacity: 0;
		pointer-events: none;
		transition: opacity 160ms ease;
	}

	.overlay-root.visible {
		opacity: 1;
	}

	.header {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.5rem 1rem;
		font-family: 'IBM Plex Mono', ui-monospace, monospace;
	}

	.eyebrow {
		font-size: 0.65rem;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.5);
	}

	.title {
		font-size: 1.15rem;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.95);
	}

	.hint {
		margin-left: auto;
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.45);
	}

	.board-wrap {
		width: 100%;
	}
</style>
