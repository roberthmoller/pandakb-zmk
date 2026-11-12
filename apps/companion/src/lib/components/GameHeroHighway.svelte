<script lang="ts">
	import SpellIcon from '#lib/components/SpellIcon.svelte';
	import {
		HIT_WINDOW,
		NOTE_TRAVEL_MS,
		typingState
	} from '#lib/state/typing.svelte';

	const done = $derived(typingState.done);
	const lanes = $derived(typingState.highwayLanes);
	const notes = $derived(typingState.visibleHighwayNotes);
	const laneCount = $derived(Math.max(lanes.length, 1));
	const freelineLive = $derived(typingState.highwayNow < typingState.freelinePulseUntil);
	const rockTone = $derived.by(() => {
		if (typingState.rock >= 70) return 'hot';
		if (typingState.rock <= 25) return 'cold';
		return 'ok';
	});
	const showPadIcons = $derived(laneCount <= 4);

	/** Gem size always fits inside one lane fraction of the stage. */
	const gemSize = $derived(Math.round(Math.max(16, Math.min(28, 120 / laneCount))));

	function keyBadge(char: string): string {
		if (char === ' ') return '␣';
		return char.toUpperCase();
	}

	function noteStyle(progress: number, lane: number) {
		const laneWidth = 100 / laneCount;
		const x = laneWidth * (lane + 0.5);
		const y = 4 + Math.max(0, Math.min(progress, 1.18)) * 84;
		const scale = 0.55 + Math.max(0, Math.min(progress, 1)) * 0.28;
		const inWindow = Math.abs(progress - 1) <= HIT_WINDOW;
		return {
			left: `${x}%`,
			top: `${y}%`,
			width: `${Math.min(laneWidth * 0.7, 100 / Math.max(laneCount, 1))}%`,
			transform: `translate(-50%, -50%) scale(${scale})`,
			zIndex: String(10 + Math.round(progress * 40)),
			'--gem-glow': inWindow ? '1' : '0'
		};
	}

	function laneLit(lane: number): boolean {
		const until = typingState.laneFlash[lane] ?? 0;
		return typingState.highwayNow < until;
	}
</script>

<div
	class="hero"
	aria-label="Guitar Hero highway"
	style:--lanes={laneCount}
	style:--travel="{NOTE_TRAVEL_MS}ms"
>
	<aside class="hud" aria-label="Performance">
		<div class="meter" class:hot={rockTone === 'hot'} class:cold={rockTone === 'cold'} title="Rock meter">
			<div class="meter-fill" style:height="{typingState.rock}%"></div>
			<span class="meter-label">Rock</span>
		</div>
		<div class="scores">
			<div>
				<div class="k">Score</div>
				<div class="v">{typingState.score}</div>
			</div>
			<div>
				<div class="k">Streak</div>
				<div class="v">
					{typingState.streak}
					<span class="mult">×{typingState.multiplier}</span>
				</div>
			</div>
		</div>
	</aside>

	<div class="stage">
		<div class="sky"></div>
		<div class="road-wrap">
			<div class="road">
				{#each lanes as lane, i (lane.char + i)}
					<div
						class="fret"
						class:lit={laneLit(i)}
						style:--fret-color={lane.color}
						style:left="{(100 / laneCount) * i}%"
						style:width="{100 / laneCount}%"
					>
						<div class="fret-groove"></div>
					</div>
				{/each}

				<div class="freeline" class:live={freelineLive}></div>

				{#each notes as note (note.id + note.state)}
					{@const bind = typingState.bindForChar(note.char)}
					{@const style = noteStyle(note.progress, note.lane)}
					<div
						class="gem"
						class:active={note.state === 'active'}
						style:left={style.left}
						style:top={style.top}
						style:width={style.width}
						style:transform={style.transform}
						style:z-index={style.zIndex}
						style:--gem-color={bind?.color ?? lanes[note.lane]?.color ?? '#aaa'}
						style:--gem-glow={style['--gem-glow']}
					>
						<div class="gem-fx" class:hit={note.state === 'hit'} class:miss={note.state === 'miss'}>
							{#if note.state === 'hit'}
								<span class="burst" aria-hidden="true"></span>
							{/if}
							{#if bind}
								<SpellIcon
									symbol={bind.symbol}
									color={bind.color}
									badge={keyBadge(note.char)}
									size={gemSize}
									waiting={note.state === 'active' && Math.abs(note.progress - 1) <= HIT_WINDOW}
									hit={note.state === 'hit'}
									dim={note.state === 'miss'}
								/>
							{:else}
								<span class="fallback" style:font-size="{Math.max(10, gemSize * 0.45)}px"
									>{keyBadge(note.char)}</span
								>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>

		<div class="bridge">
			{#each lanes as lane, i (lane.char + '-pad')}
				<div
					class="pad"
					class:lit={laneLit(i)}
					style:--fret-color={lane.color}
					title={lane.bind?.action ?? keyBadge(lane.char)}
				>
					{#if showPadIcons && lane.bind}
						<SpellIcon symbol={lane.bind.symbol} color={lane.color} size={16} />
					{/if}
					<span class="pad-key">{keyBadge(lane.char)}</span>
				</div>
			{/each}
		</div>

		{#if done}
			<div class="complete">
				<p class="font-mono text-sm uppercase tracking-widest text-muted-foreground">Chart clear</p>
				<p class="mt-1 text-lg text-foreground">
					{typingState.score} pts · best streak {typingState.bestStreak}
				</p>
				<p class="mt-1 text-sm text-muted-foreground">Space or Enter for the next drill</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.hero {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		gap: 0.75rem;
		align-items: stretch;
		min-height: 36rem;
		width: 100%;
		max-width: 100%;
	}

	.hud {
		display: flex;
		flex-direction: column;
		gap: 0.85rem;
		width: 5.5rem;
		flex: 0 0 auto;
		padding-top: 0.5rem;
	}

	.meter {
		position: relative;
		flex: 1;
		min-height: 14rem;
		overflow: hidden;
		border-radius: 999px;
		background: #141414;
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08);
	}

	.meter-fill {
		position: absolute;
		right: 0;
		bottom: 0;
		left: 0;
		background: linear-gradient(180deg, #e5e5e5 0%, #737373 100%);
		transition: height 120ms linear;
	}

	.meter.hot .meter-fill {
		background: linear-gradient(180deg, #fafafa 0%, #fbbf24 55%, #f97316 100%);
		box-shadow: 0 0 18px rgba(251, 191, 36, 0.35);
	}

	.meter.cold .meter-fill {
		background: linear-gradient(180deg, #64748b 0%, #ef4444 100%);
	}

	.meter-label {
		position: absolute;
		bottom: 0.55rem;
		left: 50%;
		translate: -50% 0;
		font-family: var(--font-mono), ui-monospace, monospace;
		font-size: 0.58rem;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: #0a0a0a;
		mix-blend-mode: difference;
	}

	.scores {
		display: grid;
		gap: 0.65rem;
	}

	.k {
		font-family: var(--font-mono), ui-monospace, monospace;
		font-size: 0.58rem;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--muted-foreground);
	}

	.v {
		font-size: 1.35rem;
		font-weight: 600;
		letter-spacing: -0.02em;
		line-height: 1.1;
	}

	.mult {
		margin-left: 0.2rem;
		font-size: 0.85rem;
		color: var(--muted-foreground);
	}

	.stage {
		position: relative;
		overflow: hidden;
		min-width: 0;
		width: 100%;
		border-radius: 0.85rem;
		background: #090909;
		box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
		min-height: 36rem;
	}

	.sky {
		position: absolute;
		inset: 0 0 48% 0;
		background:
			radial-gradient(ellipse 70% 80% at 50% 100%, rgba(255, 255, 255, 0.07), transparent 60%),
			linear-gradient(180deg, #121212 0%, #0a0a0a 100%);
	}

	.road-wrap {
		position: absolute;
		inset: 2% 0 13% 0;
		overflow: hidden;
		perspective: 680px;
		perspective-origin: 50% 0%;
	}

	.road {
		position: absolute;
		inset: 0 8% 0 8%;
		transform-origin: 50% 0%;
		/* No scale-up — that was pushing outer frets off-screen. */
		transform: rotateX(50deg);
		overflow: hidden;
		background:
			linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.08)),
			repeating-linear-gradient(
				90deg,
				transparent 0,
				transparent calc(100% / var(--lanes) - 1px),
				rgba(255, 255, 255, 0.1) calc(100% / var(--lanes) - 1px),
				rgba(255, 255, 255, 0.1) calc(100% / var(--lanes))
			),
			linear-gradient(180deg, #1a1a1a 0%, #101010 55%, #171717 100%);
		border: 1px solid rgba(255, 255, 255, 0.08);
		box-shadow:
			0 0 40px rgba(0, 0, 0, 0.65),
			inset 0 0 30px rgba(0, 0, 0, 0.35);
	}

	.fret {
		position: absolute;
		inset: 0 auto 0 0;
		pointer-events: none;
		overflow: hidden;
	}

	.fret-groove {
		position: absolute;
		inset: 0 38%;
		background: linear-gradient(
			180deg,
			transparent,
			color-mix(in srgb, var(--fret-color) 22%, transparent) 40%,
			color-mix(in srgb, var(--fret-color) 55%, transparent)
		);
		opacity: 0.55;
	}

	.fret.lit .fret-groove {
		opacity: 1;
		filter: drop-shadow(0 0 8px var(--fret-color));
	}

	.freeline {
		position: absolute;
		right: 0;
		bottom: 12%;
		left: 0;
		z-index: 5;
		height: 3px;
		background: rgba(255, 255, 255, 0.35);
		box-shadow: 0 0 12px rgba(255, 255, 255, 0.15);
	}

	.freeline.live {
		background: #fff;
		box-shadow:
			0 0 18px rgba(255, 255, 255, 0.55),
			0 0 40px rgba(255, 255, 255, 0.2);
	}

	.gem {
		position: absolute;
		display: grid;
		place-items: center;
		aspect-ratio: 1;
		max-width: 1.85rem;
		overflow: hidden;
		pointer-events: none;
		filter: drop-shadow(0 4px 8px color-mix(in srgb, var(--gem-color) 22%, transparent));
		transition: filter 80ms ease;
	}

	.gem.active {
		filter: drop-shadow(
			0 0 calc(var(--gem-glow) * 10px) color-mix(in srgb, var(--gem-color) 65%, transparent)
		);
	}

	.gem-fx {
		position: relative;
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
	}

	.gem-fx :global(.spell) {
		width: 100% !important;
		height: 100% !important;
		max-width: 100%;
		max-height: 100%;
	}

	.gem-fx :global(.badge) {
		font-size: 0.45rem;
		right: 1px;
		bottom: 1px;
	}

	.gem-fx.hit {
		animation: shatter 280ms ease forwards;
	}

	.gem-fx.miss {
		animation: fallout 300ms ease forwards;
	}

	.burst {
		position: absolute;
		inset: -12%;
		border-radius: 999px;
		background: radial-gradient(circle, color-mix(in srgb, var(--gem-color) 80%, white), transparent 70%);
		animation: burst 280ms ease forwards;
	}

	.fallback {
		display: grid;
		place-items: center;
		width: 100%;
		height: 100%;
		border-radius: 0.4rem;
		background: #222;
		font-family: var(--font-mono), ui-monospace, monospace;
	}

	.bridge {
		position: absolute;
		right: 0;
		bottom: 0.4rem;
		left: 0;
		z-index: 6;
		display: grid;
		grid-template-columns: repeat(var(--lanes), minmax(0, 1fr));
		gap: 2px;
		width: 100%;
		padding: 0 0.35rem;
		box-sizing: border-box;
	}

	.pad {
		display: grid;
		place-items: center;
		gap: 0;
		min-width: 0;
		width: 100%;
		min-height: 2.15rem;
		padding: 0.15rem 0;
		overflow: hidden;
		border: 1px solid color-mix(in srgb, var(--fret-color) 40%, transparent);
		border-radius: 0.35rem;
		background: linear-gradient(180deg, #1d1d1d, #121212);
		color: var(--fret-color);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
	}

	.pad.lit {
		background: linear-gradient(
			180deg,
			color-mix(in srgb, var(--fret-color) 35%, #222),
			color-mix(in srgb, var(--fret-color) 12%, #111)
		);
		box-shadow:
			0 0 12px color-mix(in srgb, var(--fret-color) 35%, transparent),
			inset 0 0 0 1px color-mix(in srgb, var(--fret-color) 50%, transparent);
	}

	.pad-key {
		font-family: var(--font-mono), ui-monospace, monospace;
		font-size: clamp(0.5rem, 1.8vw, 0.7rem);
		font-weight: 600;
		letter-spacing: 0.04em;
		color: #f5f5f5;
		line-height: 1;
	}

	.complete {
		position: absolute;
		inset: 0;
		z-index: 8;
		display: grid;
		place-content: center;
		text-align: center;
		background: rgba(0, 0, 0, 0.55);
		backdrop-filter: blur(4px);
	}

	@keyframes shatter {
		0% {
			opacity: 1;
			transform: scale(1);
			filter: brightness(1.4);
		}
		100% {
			opacity: 0;
			transform: scale(1.35);
			filter: brightness(2);
		}
	}

	@keyframes fallout {
		0% {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
		100% {
			opacity: 0.12;
			filter: grayscale(1);
			transform: translateY(12px) scale(0.72);
		}
	}

	@keyframes burst {
		from {
			opacity: 0.9;
			transform: scale(0.4);
		}
		to {
			opacity: 0;
			transform: scale(1.35);
		}
	}

	@media (max-width: 720px) {
		.hero {
			grid-template-columns: minmax(0, 1fr);
			min-height: 30rem;
		}

		.stage {
			min-height: 30rem;
		}

		.hud {
			flex-direction: row;
			width: auto;
			align-items: stretch;
		}

		.meter {
			min-height: 3.5rem;
			width: 3.2rem;
			flex: 0 0 auto;
		}

		.scores {
			grid-auto-flow: column;
			align-content: center;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.gem-fx.hit,
		.gem-fx.miss,
		.burst,
		.meter-fill {
			animation: none;
			transition: none;
		}
	}
</style>
