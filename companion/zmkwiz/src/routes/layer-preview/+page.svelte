<script lang="ts">
	import KeyboardViz from '#lib/components/KeyboardViz.svelte';
	import { boardState } from '#lib/state/board.svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';

	type PreviewEvent = { show: boolean; layer: number };

	let visible = $state(false);
	let layerName = $state('Layer');

	onMount(() => {
		let unlisten: (() => void) | undefined;

		void (async () => {
			try {
				unlisten = await listen<PreviewEvent>('companion://layer-preview', (event) => {
					const { show, layer } = event.payload;
					if (show) {
						boardState.setFirmwarePreview(layer);
						layerName = boardState.activeLayer?.name ?? `Layer ${layer}`;
						visible = true;
					} else {
						boardState.setFirmwarePreview(null);
						visible = false;
					}
				});
			} catch {
				// Browser / non-Tauri preview — stay idle.
			}
		})();

		return () => unlisten?.();
	});
</script>

<div class="overlay-root" class:visible aria-hidden={!visible}>
	<div class="glass">
		<header class="header">
			<span class="eyebrow">Layer preview</span>
			<strong class="title">{layerName}</strong>
			<span class="hint">Release the layer key to dismiss</span>
		</header>
		<div class="board-wrap">
			<KeyboardViz />
		</div>
	</div>
</div>

<style>
	:global(html),
	:global(body) {
		background: transparent !important;
		overflow: hidden;
	}

	.overlay-root {
		box-sizing: border-box;
		display: flex;
		min-height: 100vh;
		align-items: center;
		justify-content: center;
		padding: 1.25rem;
		opacity: 0;
		pointer-events: none;
		transition: opacity 160ms ease;
	}

	.overlay-root.visible {
		opacity: 1;
	}

	.glass {
		width: min(940px, 100%);
		border: 1px solid rgba(255, 255, 255, 0.14);
		border-radius: 1.25rem;
		background: linear-gradient(
			165deg,
			rgba(28, 28, 30, 0.72) 0%,
			rgba(12, 12, 14, 0.55) 100%
		);
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.45),
			inset 0 1px 0 rgba(255, 255, 255, 0.12);
		-webkit-backdrop-filter: blur(28px) saturate(1.4);
		backdrop-filter: blur(28px) saturate(1.4);
		padding: 1rem 1.25rem 1.25rem;
	}

	.header {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 0.5rem 1rem;
		margin-bottom: 0.75rem;
		font-family: 'IBM Plex Mono', ui-monospace, monospace;
	}

	.eyebrow {
		font-size: 0.65rem;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: rgba(255, 255, 255, 0.45);
	}

	.title {
		font-size: 1.15rem;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.92);
	}

	.hint {
		margin-left: auto;
		font-size: 0.7rem;
		color: rgba(255, 255, 255, 0.4);
	}

	.board-wrap {
		width: 100%;
	}
</style>
