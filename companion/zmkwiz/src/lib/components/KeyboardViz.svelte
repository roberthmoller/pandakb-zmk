<script lang="ts">
	import KeyCap from '#lib/components/KeyCap.svelte';
	import { isMomentaryLayer } from '#lib/keymap/momentary';
	import { bindingAt } from '#lib/keymap/resolve';
	import type { PhysicalKey } from '#lib/keymap/types';
	import { boardState } from '#lib/state/board.svelte';
	import { testState } from '#lib/state/test.svelte';
	import { typingState } from '#lib/state/typing.svelte';
	import { onMount } from 'svelte';

	type Props = {
		training?: boolean;
		testing?: boolean;
	};

	type Point = [number, number];

	let { training = false, testing = false }: Props = $props();

	const pad = 16;
	const maxUnit = 88;
	const minUnit = 32;

	let vizEl: HTMLDivElement | undefined = $state();
	let unit = $state(64);

	function keyCorners(key: PhysicalKey): Point[] {
		const local: Point[] = [
			[key.x, key.y],
			[key.x + key.w, key.y],
			[key.x + key.w, key.y + key.h],
			[key.x, key.y + key.h]
		];

		if (!key.r) return local;

		const rad = (key.r * Math.PI) / 180;
		const cos = Math.cos(rad);
		const sin = Math.sin(rad);

		return local.map(([cx, cy]) => {
			const dx = cx - key.rx;
			const dy = cy - key.ry;
			return [key.rx + dx * cos - dy * sin, key.ry + dx * sin + dy * cos];
		});
	}

	function keyBounds(key: PhysicalKey) {
		const corners = keyCorners(key);
		let minX = Infinity;
		let minY = Infinity;
		let maxX = -Infinity;
		let maxY = -Infinity;
		for (const [x, y] of corners) {
			minX = Math.min(minX, x);
			minY = Math.min(minY, y);
			maxX = Math.max(maxX, x);
			maxY = Math.max(maxY, y);
		}
		return { minX, minY, maxX, maxY };
	}

	const layoutSize = $derived.by(() => {
		const keys = boardState.board.keys;
		let minX = 0;
		let minY = 0;
		let maxX = 0;
		let maxY = 0;
		for (const key of keys) {
			const bounds = keyBounds(key);
			minX = Math.min(minX, bounds.minX);
			minY = Math.min(minY, bounds.minY);
			maxX = Math.max(maxX, bounds.maxX);
			maxY = Math.max(maxY, bounds.maxY);
		}
		return {
			minX,
			minY,
			maxX,
			maxY,
			width: maxX - minX,
			height: maxY - minY
		};
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
		const next = Math.floor((width - pad * 2) / layoutSize.width);
		unit = Math.max(minUnit, Math.min(maxUnit, next));
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

	const toneVersion = $derived(
		testing
			? `${testState.suspectIndex}:${testState.held.join(',')}:${testState.seen.join(',')}:${testState.totalDowns}`
			: `${typingState.layoutId}:${typingState.hits}:${typingState.misses}:${typingState.streak}:${typingState.expectedBind?.char ?? ''}:${typingState.wrongIndex}:${typingState.hitIndex}:${typingState.wrongUntil}:${typingState.hitUntil}:${typingState.done}:${boardState.pressedIndices.join(',')}`
	);

	const gameOverlay = $derived(training && typingState.isGameLayout);
</script>

<div class="viz" class:training class:testing bind:this={vizEl}>
	<div class="board" style:width="{bounds.width}px" style:height="{bounds.height}px">
		<div class="keys" style:transform="translate({originOffset.x}px, {originOffset.y}px)">
			{#each boardState.board.keys as keyDef, order (keyDef.index)}
				{@const binding = bindingAt(
					training || testing ? boardState.baseLayer : boardState.activeLayer,
					keyDef.index
				)}
				{@const tone = training ? typingState.toneFor(keyDef.index) : null}
				{@const testTone = testing ? testState.toneFor(keyDef.index) : null}
				{@const overlay = gameOverlay ? typingState.overlayForIndex(keyDef.index) : null}
				<span class="sr-only">{toneVersion}</span>
				<KeyCap
					{keyDef}
					{binding}
					{unit}
					{tone}
					{testTone}
					{order}
					{overlay}
					overlayMuted={gameOverlay && !overlay}
					interactive={!training}
					pressed={!training && !testing && boardState.isPressed(keyDef.index)}
					selected={
						testing
							? testState.suspectIndex === keyDef.index
							: !training && boardState.selectedKeyIndex === keyDef.index
					}
					hovered={!training && !testing && boardState.hoveredKeyIndex === keyDef.index}
					layerAccent={!training && !testing && isMomentaryLayer(binding)}
					onclick={() =>
						testing ? testState.markSuspect(keyDef.index) : boardState.selectKey(keyDef.index)}
					onhover={() => {
						if (!testing) boardState.hoverKey(keyDef.index);
					}}
					onleave={() => {
						if (!testing) boardState.hoverKey(null);
					}}
					onpress={() => {
						if (!testing) boardState.holdMomentary(binding);
					}}
					onrelease={() => {
						if (!testing) boardState.releaseHold();
					}}
				/>
			{/each}
		</div>
	</div>
</div>

<style>
	.viz {
		width: 100%;
		background: transparent;
	}

	.board {
		position: relative;
		margin: 0 auto;
	}

	.keys {
		position: absolute;
		inset: 0 auto auto 0;
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		border: 0;
	}
</style>
