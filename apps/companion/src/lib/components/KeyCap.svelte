<script lang="ts">
	import SpellIcon from '#lib/components/SpellIcon.svelte';
	import type { Binding, PhysicalKey } from '#lib/keymap/types';
	import type { TestTone } from '#lib/state/test.svelte';
	import type { KeyTone } from '#lib/state/typing.svelte';
	import type { GameBind } from '#lib/typing/layouts';

	type Props = {
		keyDef: PhysicalKey;
		binding: Binding | null;
		unit: number;
		selected: boolean;
		hovered: boolean;
		layerAccent: boolean;
		pressed?: boolean;
		tone?: KeyTone;
		testTone?: TestTone | null;
		interactive?: boolean;
		/** Game-layout overlay; replaces the keymap label when set. */
		overlay?: GameBind | null;
		/** Dim keys that are unbound in the active game layout. */
		overlayMuted?: boolean;
		/** Stagger index for entrance motion */
		order?: number;
		onclick: () => void;
		onhover: () => void;
		onleave: () => void;
		onpress: () => void;
		onrelease: () => void;
	};

	let {
		keyDef,
		binding,
		unit,
		selected,
		hovered,
		layerAccent,
		pressed = false,
		tone = null,
		testTone = null,
		interactive = true,
		overlay = null,
		overlayMuted = false,
		order = 0,
		onclick,
		onhover,
		onleave,
		onpress,
		onrelease
	}: Props = $props();

	const faceLabel = $derived(overlay?.label ?? binding?.label ?? '');
	const width = $derived(keyDef.w * unit);
	const height = $derived(keyDef.h * unit);
	const left = $derived(keyDef.x * unit);
	const top = $derived(keyDef.y * unit);
	const rotate = $derived(keyDef.r ? `rotate(${keyDef.r}deg)` : 'none');
	const rotateOrigin = $derived(
		keyDef.r
			? `${(keyDef.rx - keyDef.x) * unit}px ${(keyDef.ry - keyDef.y) * unit}px`
			: 'center'
	);
	const gap = $derived(Math.max(2, Math.round(unit * 0.06)));
	const labelSize = $derived(Math.max(10, Math.min(18, Math.round(unit * 0.22))));
	const iconSize = $derived(Math.max(18, Math.min(30, Math.round(unit * 0.55))));
</script>

<button
	type="button"
	class="key"
	class:selected
	class:hovered
	class:pressed
	class:layer-accent={layerAccent}
	class:tone-expected={tone === 'expected'}
	class:tone-shift={tone === 'shift'}
	class:tone-wrong={tone === 'wrong'}
	class:tone-hit={tone === 'hit'}
	class:test-idle={testTone === 'idle'}
	class:test-seen={testTone === 'seen'}
	class:test-active={testTone === 'active'}
	class:test-suspect={testTone === 'suspect'}
	class:test-rowmate={testTone === 'rowmate'}
	class:test-colmate={testTone === 'colmate'}
	class:transparent={binding?.behavior === 'trans'}
	class:overlay-muted={overlayMuted}
	class:display-only={!interactive}
	style:left="{left}px"
	style:top="{top}px"
	style:width="{width}px"
	style:height="{height}px"
	style:padding="{gap}px"
	style:--enter-delay="{Math.min(order, 40) * 12}ms"
	aria-label={overlay?.action || binding?.label || `Key ${keyDef.index}`}
	aria-pressed={selected}
	tabindex={interactive ? 0 : -1}
	onmouseenter={onhover}
	onmouseleave={onleave}
	onpointerdown={(event) => {
		if (!interactive) {
			event.preventDefault();
			return;
		}
		onpress();
	}}
	onpointerup={onrelease}
	onpointercancel={onrelease}
	onclick={(event) => {
		if (!interactive) {
			event.preventDefault();
			return;
		}
		onclick();
	}}
>
	<span class="rotator" style:transform={rotate} style:transform-origin={rotateOrigin}>
		<span class="shell">
			<span class="face">
				{#if overlay}
					<span class="spell-slot">
						<SpellIcon
							symbol={overlay.symbol}
							color={overlay.color}
							size={iconSize}
							waiting={tone === 'expected'}
							hit={tone === 'hit'}
						/>
					</span>
				{:else}
					<span class="label" style:font-size="{labelSize}px">{faceLabel}</span>
				{/if}
			</span>
		</span>
	</span>
</button>

<style>
	.key {
		position: absolute;
		display: block;
		box-sizing: border-box;
		margin: 0;
		border: 0;
		background: transparent;
		color: var(--key-text);
		cursor: pointer;
		appearance: none;
		-webkit-tap-highlight-color: transparent;
		animation: key-enter 480ms cubic-bezier(0.22, 1, 0.36, 1) both;
		animation-delay: var(--enter-delay, 0ms);
	}

	.rotator {
		display: block;
		width: 100%;
		height: 100%;
	}

	.key:focus-visible .shell {
		box-shadow:
			0 0 0 2px var(--background),
			0 0 0 3px color-mix(in srgb, var(--key-accent) 70%, transparent);
	}

	.shell {
		display: block;
		width: 100%;
		height: 100%;
		border-radius: calc(var(--radius) + 2px);
		background: linear-gradient(165deg, #2c2c2c 0%, #1a1a1a 55%, #151515 100%);
		box-shadow:
			0 1px 0 rgba(255, 255, 255, 0.05),
			0 6px 14px rgba(0, 0, 0, 0.38),
			inset 0 1px 0 rgba(255, 255, 255, 0.07);
		transform: translateZ(0);
		transition:
			transform 140ms cubic-bezier(0.22, 1, 0.36, 1),
			box-shadow 180ms ease,
			background 180ms ease,
			filter 180ms ease;
	}

	.face {
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		border-radius: calc(var(--radius) - 1px);
		background: linear-gradient(180deg, #262626 0%, var(--key-face) 42%, #1d1d1d 100%);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		transition:
			background 180ms ease,
			box-shadow 180ms ease,
			color 140ms ease;
	}

	.key:hover .shell,
	.key.hovered .shell {
		transform: translateY(-1px);
		box-shadow:
			0 1px 0 rgba(255, 255, 255, 0.07),
			0 10px 22px rgba(0, 0, 0, 0.42),
			inset 0 1px 0 rgba(255, 255, 255, 0.1);
		filter: brightness(1.06);
	}

	.key.selected .shell,
	.key.layer-accent .shell {
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--key-accent) 55%, transparent),
			0 0 18px color-mix(in srgb, var(--key-accent) 22%, transparent),
			0 8px 18px rgba(0, 0, 0, 0.4),
			inset 0 1px 0 rgba(255, 255, 255, 0.12);
	}

	.key.selected .face,
	.key.layer-accent .face {
		background: linear-gradient(180deg, #323232 0%, #262626 100%);
	}

	.key.pressed .shell {
		transform: translateY(1px) scale(0.96);
		box-shadow:
			0 0 0 1px color-mix(in srgb, #f5f5f5 40%, transparent),
			0 0 20px color-mix(in srgb, #f5f5f5 18%, transparent),
			0 2px 6px rgba(0, 0, 0, 0.45),
			inset 0 2px 6px rgba(0, 0, 0, 0.35);
		filter: none;
	}

	.key.pressed .face {
		background: linear-gradient(180deg, #f2f2f2 0%, #e4e4e4 100%);
		color: #0f0f0f;
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.12);
	}

	.key.tone-expected .shell {
		animation: expect-pulse 1.4s ease-in-out infinite;
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--key-expected) 70%, transparent),
			0 0 22px color-mix(in srgb, var(--key-expected) 28%, transparent),
			0 6px 14px rgba(0, 0, 0, 0.35);
	}

	.key.tone-expected .face {
		background: linear-gradient(180deg, #ececec 0%, var(--key-expected-face) 100%);
		color: #0f0f0f;
	}

	.key.tone-shift .shell {
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--key-shift) 60%, transparent),
			0 0 16px color-mix(in srgb, var(--key-shift) 24%, transparent),
			0 6px 14px rgba(0, 0, 0, 0.35);
	}

	.key.tone-shift .face {
		background: linear-gradient(180deg, #6b6b6b 0%, var(--key-shift-face) 100%);
		color: #f5f5f5;
	}

	.key.tone-wrong .shell {
		animation: shake 200ms ease;
		box-shadow:
			0 0 0 1px color-mix(in srgb, #fafafa 50%, transparent),
			0 0 18px color-mix(in srgb, #fafafa 16%, transparent),
			0 4px 10px rgba(0, 0, 0, 0.4);
	}

	.key.tone-wrong .face {
		background: linear-gradient(180deg, #6a6a6a 0%, #454545 100%);
		color: #fafafa;
	}

	.key.tone-hit .shell {
		animation: hit-flash 320ms ease;
		box-shadow:
			0 0 0 1px color-mix(in srgb, #f5f5f5 65%, transparent),
			0 0 24px color-mix(in srgb, #f5f5f5 30%, transparent),
			0 6px 14px rgba(0, 0, 0, 0.35);
	}

	.key.tone-hit .face {
		background: linear-gradient(180deg, #f7f7f7 0%, #ececec 100%);
		color: #0f0f0f;
	}

	.key.test-idle {
		opacity: 0.42;
	}

	.key.test-seen .shell {
		opacity: 1;
		box-shadow:
			0 0 0 1px color-mix(in srgb, #a3a3a3 55%, transparent),
			0 6px 14px rgba(0, 0, 0, 0.35);
	}

	.key.test-seen .face {
		background: linear-gradient(180deg, #454545 0%, #333 100%);
	}

	.key.test-active .shell {
		opacity: 1;
		transform: translateY(1px) scale(0.96);
		box-shadow:
			0 0 0 1px color-mix(in srgb, #f5f5f5 55%, transparent),
			0 0 20px color-mix(in srgb, #f5f5f5 20%, transparent),
			0 2px 6px rgba(0, 0, 0, 0.45);
	}

	.key.test-active .face {
		background: linear-gradient(180deg, #f2f2f2 0%, #e4e4e4 100%);
		color: #0f0f0f;
	}

	.key.test-suspect .shell {
		opacity: 1;
		box-shadow:
			0 0 0 2px #fafafa,
			0 0 22px color-mix(in srgb, #fafafa 22%, transparent),
			0 8px 18px rgba(0, 0, 0, 0.4);
	}

	.key.test-suspect .face {
		background: linear-gradient(180deg, #1f1f1f 0%, #121212 100%);
		box-shadow: inset 0 0 0 1px #fafafa;
	}

	.key.test-rowmate,
	.key.test-colmate {
		opacity: 0.78;
	}

	.key.test-rowmate .shell,
	.key.test-colmate .shell {
		box-shadow:
			0 0 0 1px color-mix(in srgb, #737373 50%, transparent),
			0 6px 14px rgba(0, 0, 0, 0.3);
		filter: saturate(0.7);
	}

	.key.display-only {
		cursor: default;
	}

	.key.transparent .label {
		color: var(--key-muted);
	}

	.spell-slot {
		display: grid;
		place-items: center;
		max-width: 100%;
		max-height: 100%;
		overflow: hidden;
	}

	.spell-slot :global(.spell) {
		filter: none;
	}

	.spell-slot :global(.badge) {
		display: none;
	}

	.key.overlay-muted {
		opacity: 0.34;
	}

	.key.overlay-muted.tone-expected,
	.key.overlay-muted.tone-shift,
	.key.overlay-muted.tone-wrong,
	.key.overlay-muted.tone-hit {
		opacity: 1;
	}

	.label {
		font-family: var(--font-mono), ui-monospace, monospace;
		font-weight: 500;
		letter-spacing: 0.01em;
		line-height: 1.1;
		text-align: center;
		overflow: hidden;
		max-width: 100%;
		word-break: break-word;
		opacity: 0.92;
	}

	@keyframes key-enter {
		from {
			opacity: 0;
			filter: blur(2px);
			transform: translateY(6px) scale(0.94);
		}
		to {
			opacity: 1;
			filter: none;
			transform: none;
		}
	}

	@keyframes expect-pulse {
		0%,
		100% {
			filter: brightness(1);
		}
		50% {
			filter: brightness(1.12);
		}
	}

	@keyframes hit-flash {
		0% {
			filter: brightness(1.25);
		}
		100% {
			filter: brightness(1);
		}
	}

	@keyframes shake {
		0%,
		100% {
			translate: 0 0;
		}
		25% {
			translate: -2px 0;
		}
		75% {
			translate: 2px 0;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.key,
		.key.tone-expected .shell,
		.key.tone-wrong .shell,
		.key.tone-hit .shell {
			animation: none;
		}

		.shell,
		.face {
			transition: none;
		}
	}
</style>
