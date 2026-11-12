<script lang="ts">
	import type { SpellSymbol } from '#lib/typing/layouts';

	type Props = {
		symbol: SpellSymbol;
		color?: string;
		badge?: string;
		size?: number;
		waiting?: boolean;
		hit?: boolean;
		dim?: boolean;
	};

	let {
		symbol,
		color = '#a3a3a3',
		badge = '',
		size = 56,
		waiting = false,
		hit = false,
		dim = false
	}: Props = $props();
</script>

<span
	class="spell"
	class:waiting
	class:hit
	class:dim
	style:--spell-color={color}
	style:width="{size}px"
	style:height="{size}px"
	aria-hidden="true"
>
	<span class="frame">
		<svg class="glyph" viewBox="0 0 24 24" fill="none">
			{#if symbol === 'gem'}
				<path
					d="M12 3 19 9.5 12 21 5 9.5Z"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
				<path d="M5 9.5h14" stroke="currentColor" stroke-width="1.6" />
			{:else if symbol === 'arrow-up'}
				<path
					d="M12 19V6M7 10l5-5 5 5"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'arrow-down'}
				<path
					d="M12 5v13M7 14l5 5 5-5"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'arrow-left'}
				<path
					d="M19 12H6M10 7l-5 5 5 5"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'arrow-right'}
				<path
					d="M5 12h13M14 7l5 5-5 5"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'strafe-left'}
				<path
					d="M18 7H8l-3 5 3 5h10"
					stroke="currentColor"
					stroke-width="1.7"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'strafe-right'}
				<path
					d="M6 7h10l3 5-3 5H6"
					stroke="currentColor"
					stroke-width="1.7"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'jump'}
				<path
					d="M6 18c2-5 4-8 6-8s4 3 6 8M12 10V5"
					stroke="currentColor"
					stroke-width="1.7"
					stroke-linecap="round"
				/>
			{:else if symbol === 'hand'}
				<path
					d="M8 11V8.5a1.5 1.5 0 0 1 3 0V11m0 0V7.5a1.5 1.5 0 0 1 3 0V11m0 0V8.5a1.5 1.5 0 0 1 3 0V14c0 3-2 5-5 5h-1c-3 0-5-2.2-5-5v-3.5A1.5 1.5 0 0 1 8 11"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'swords'}
				<path
					d="m6.5 17.5 8-8M9 7l1.5-1.5L18 13l-1.5 1.5M15 7l1.5-1.5L20 9.5 18.5 11M8.5 15.5 7 17l-2-.5.5-2 1.5-1.5"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
				/>
			{:else if symbol === 'chat'}
				<path
					d="M5 16.5V7.8A1.8 1.8 0 0 1 6.8 6h10.4A1.8 1.8 0 0 1 19 7.8v5.4a1.8 1.8 0 0 1-1.8 1.8H9l-4 2.5Z"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'sit'}
				<path
					d="M7 18V10h10v8M7 13h10M9 10V7h6v3"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linecap="round"
				/>
			{:else if symbol === 'sheath'}
				<path
					d="M8 5h8l1 4H7l1-4ZM9 9v9h6V9"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'bag'}
				<path
					d="M7 9h10l1 11H6L7 9Zm3 0V7.5A2.5 2.5 0 0 1 12.5 5h0A2.5 2.5 0 0 1 15 7.5V9"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'person'}
				<circle cx="12" cy="8" r="2.4" stroke="currentColor" stroke-width="1.6" />
				<path
					d="M6.5 19c1.2-3.2 3-4.8 5.5-4.8S16.3 15.8 17.5 19"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linecap="round"
				/>
			{:else if symbol === 'book'}
				<path
					d="M6 6.5A2.5 2.5 0 0 1 8.5 4H18v14H8.5A2.5 2.5 0 0 0 6 20.5Z"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
				<path d="M6 6.5V20.5" stroke="currentColor" stroke-width="1.6" />
			{:else if symbol === 'star'}
				<path
					d="m12 4 1.8 4.6L19 10l-3.5 3.2L16.5 19 12 16.4 7.5 19l1-5.8L5 10l5.2-1.4Z"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'map'}
				<path
					d="m4 7 5-2 6 2 5-2v14l-5 2-6-2-5 2Z"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linejoin="round"
				/>
				<path d="M9 5v14M15 7v14" stroke="currentColor" stroke-width="1.5" />
			{:else if symbol === 'scroll'}
				<path
					d="M8 6h9a2 2 0 0 1 2 2v9H9a2 2 0 0 0-2 2V8a2 2 0 0 1 2-2Z"
					stroke="currentColor"
					stroke-width="1.6"
				/>
				<path d="M10 10h6M10 13h5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
			{:else if symbol === 'users'}
				<circle cx="9" cy="9" r="2" stroke="currentColor" stroke-width="1.5" />
				<circle cx="16" cy="10" r="1.7" stroke="currentColor" stroke-width="1.5" />
				<path
					d="M4.5 18c.8-2.4 2.2-3.6 4.5-3.6s3.7 1.2 4.5 3.6M13 14.5c1.5 0 2.7.7 3.5 2.5"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
				/>
			{:else if symbol === 'eye'}
				<path
					d="M2.5 12S6 6.5 12 6.5 21.5 12 21.5 12 18 17.5 12 17.5 2.5 12 2.5 12Z"
					stroke="currentColor"
					stroke-width="1.5"
				/>
				<circle cx="12" cy="12" r="2.2" stroke="currentColor" stroke-width="1.5" />
			{:else if symbol === 'ability'}
				<circle cx="12" cy="12" r="7" stroke="currentColor" stroke-width="1.6" />
				<path
					d="M12 7.5v9M8.5 12h7"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linecap="round"
				/>
			{:else if symbol === 'ult'}
				<path
					d="M12 4 14.5 10.5 21 11.2 16 15.5 17.5 22 12 18.6 6.5 22 8 15.5 3 11.2l6.5-.7Z"
					stroke="currentColor"
					stroke-width="1.4"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'summoner'}
				<path
					d="M12 4c3 3 4.5 5.5 4.5 8.2A4.5 4.5 0 0 1 12 20a4.5 4.5 0 0 1-4.5-7.8C7.5 9.5 9 7 12 4Z"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'potion'}
				<path
					d="M10 4h4v3l3 6a4 4 0 0 1-3.5 6h-3A4 4 0 0 1 7 13l3-6V4Z"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linejoin="round"
				/>
				<path d="M9 14h6" stroke="currentColor" stroke-width="1.5" />
			{:else if symbol === 'trinket'}
				<circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5" />
				<path
					d="M12 4v3M12 17v3M4 12h3M17 12h3M6.5 6.5l2 2M15.5 15.5l2 2M17.5 6.5l-2 2M8.5 15.5l-2 2"
					stroke="currentColor"
					stroke-width="1.4"
					stroke-linecap="round"
				/>
			{:else if symbol === 'portal'}
				<ellipse cx="12" cy="12" rx="6" ry="8" stroke="currentColor" stroke-width="1.6" />
				<ellipse cx="12" cy="12" rx="2.5" ry="5" stroke="currentColor" stroke-width="1.4" />
			{:else if symbol === 'crosshair'}
				<circle cx="12" cy="12" r="5.5" stroke="currentColor" stroke-width="1.5" />
				<path
					d="M12 3v4M12 17v4M3 12h4M17 12h4"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
				/>
			{:else if symbol === 'stop'}
				<rect
					x="7"
					y="7"
					width="10"
					height="10"
					rx="1.5"
					stroke="currentColor"
					stroke-width="1.6"
				/>
			{:else if symbol === 'camera'}
				<path
					d="M4 9.5A1.5 1.5 0 0 1 5.5 8H9l1.2-1.8h3.6L15 8h3.5A1.5 1.5 0 0 1 20 9.5v7A1.5 1.5 0 0 1 18.5 18h-13A1.5 1.5 0 0 1 4 16.5Z"
					stroke="currentColor"
					stroke-width="1.5"
				/>
				<circle cx="12" cy="13" r="2.5" stroke="currentColor" stroke-width="1.5" />
			{:else if symbol === 'lock'}
				<rect
					x="7"
					y="11"
					width="10"
					height="8"
					rx="1.5"
					stroke="currentColor"
					stroke-width="1.6"
				/>
				<path
					d="M9 11V8.5a3 3 0 0 1 6 0V11"
					stroke="currentColor"
					stroke-width="1.6"
					stroke-linecap="round"
				/>
			{:else if symbol === 'shop'}
				<path
					d="M5 9h14l-1 10H6L5 9Zm1 0 2-4h8l2 4"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linejoin="round"
				/>
			{:else if symbol === 'chart'}
				<path
					d="M5 19V9M10 19V5M15 19v-7M20 19V8"
					stroke="currentColor"
					stroke-width="1.7"
					stroke-linecap="round"
				/>
			{:else}
				<!-- ping -->
				<circle cx="12" cy="12" r="2" fill="currentColor" />
				<circle cx="12" cy="12" r="5" stroke="currentColor" stroke-width="1.4" />
				<circle cx="12" cy="12" r="8" stroke="currentColor" stroke-width="1.2" opacity="0.55" />
			{/if}
		</svg>
		{#if badge}
			<span class="badge">{badge}</span>
		{/if}
	</span>
</span>

<style>
	.spell {
		display: inline-grid;
		place-items: center;
		color: var(--spell-color);
		filter: drop-shadow(0 6px 14px color-mix(in srgb, var(--spell-color) 28%, transparent));
		transition:
			transform 160ms ease,
			filter 160ms ease,
			opacity 160ms ease;
	}

	.spell.dim {
		opacity: 0.42;
		filter: none;
	}

	.spell.waiting {
		transform: scale(1.08);
		filter: drop-shadow(0 0 18px color-mix(in srgb, var(--spell-color) 55%, transparent));
	}

	.spell.hit {
		transform: scale(1.16);
		filter: drop-shadow(0 0 22px color-mix(in srgb, #f5f5f5 50%, transparent));
	}

	.frame {
		position: relative;
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		border-radius: 14px;
		background:
			radial-gradient(circle at 30% 25%, color-mix(in srgb, var(--spell-color) 35%, #fff) 0%, transparent 42%),
			linear-gradient(160deg, #2a2a2a 0%, #141414 100%);
		box-shadow:
			inset 0 0 0 1px color-mix(in srgb, var(--spell-color) 55%, transparent),
			inset 0 1px 0 rgba(255, 255, 255, 0.12),
			0 0 0 1px rgba(0, 0, 0, 0.45);
	}

	.glyph {
		width: 58%;
		height: 58%;
	}

	.badge {
		position: absolute;
		right: 4px;
		bottom: 3px;
		font-family: var(--font-mono), ui-monospace, monospace;
		font-size: 0.62rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		line-height: 1;
		color: #f5f5f5;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
	}

	@media (prefers-reduced-motion: reduce) {
		.spell {
			transition: none;
		}
	}
</style>
