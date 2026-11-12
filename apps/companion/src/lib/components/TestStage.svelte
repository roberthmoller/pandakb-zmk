<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { testState } from '#lib/state/test.svelte';

	const last = $derived(testState.lastEvent);
	const suspect = $derived(testState.suspect);
	const diagnosis = $derived(testState.diagnosis);
	const isMo = $derived(testState.suspectIsMomentary);
	const targetId = $derived(testState.suspectTargetLayerId);

	const diagnosisCopy = $derived.by(() => {
		switch (diagnosis) {
			case 'mo-probe':
				return targetId === 'adjust'
					? 'Stock PandaKB: L2 opens Adjust, not Raise. Right-hand letters are transparent — L2+J typing j is expected.'
					: 'Hold L2, then tap a key that only exists on that layer.';
			case 'mo-ok':
				return 'Layer HID from the probe matched — the L2 switch is closing the matrix.';
			case 'mo-dead':
				return targetId === 'adjust'
					? 'Try L2+W (RGB H− on Adjust). If underglow never changes and neighbors work, probe the switch.'
					: 'No layer HID from that probe. If neighbors work, probe the L2 switch/diode.';
			case 'working':
				return 'This key is registering. If it still feels wrong in apps, check the binding / layer — not the switch.';
			case 'row':
				return 'Same matrix row looks dead. Probe the row diode/trace before swapping the switch.';
			case 'col':
				return 'Same matrix column looks dead. Probe the column pin/trace before swapping the switch.';
			case 'single':
				return 'Neighbors look fine so far. Likely the switch, its solder joints, or its diode.';
			default:
				return 'Mark the dead key (Right L2 is preselected), then run the probe below.';
		}
	});
</script>

<section class="space-y-3" aria-label="Key tester">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				Hardware test
			</div>
			<p class="mt-1 max-w-2xl text-sm text-muted-foreground">
				{#if isMo && targetId === 'adjust'}
					<strong class="text-foreground">Your flashed keymap has no Raise.</strong>
					<code class="text-foreground">&amp;mo 2</code> activates Adjust (layer 2). Almost every
					right-hand key there is <code class="text-foreground">&amp;trans</code>, so L2+J still
					types j — that does not mean the switch is dead.
				{:else if isMo}
					<strong class="text-foreground">L2 never types by itself</strong> — ZMK
					<code class="text-foreground">&amp;mo</code> only opens a layer.
				{:else}
					Click the suspect key, then press keys. Lit = host saw HID.
				{/if}
			</p>
		</div>
		<div class="flex flex-wrap gap-2">
			<Button variant="outline" size="sm" onclick={() => testState.markRightRaise()}>
				Right L2
			</Button>
			<Button variant="outline" size="sm" onclick={() => testState.reset()}>Reset hits</Button>
		</div>
	</div>

	<div class="rounded-lg bg-[#171717] px-5 py-6 md:px-6 md:py-8" aria-live="polite">
		{#if isMo && targetId === 'adjust'}
			<div class="font-mono text-2xl tracking-tight md:text-3xl">L2+J → j is expected</div>
			<p class="mt-3 max-w-2xl text-sm text-muted-foreground">
				To verify the switch: hold <strong class="text-foreground">L2</strong> and tap
				<strong class="text-foreground">W</strong> (Adjust: RGB hue −) or
				<strong class="text-foreground">S</strong> (brightness −). Underglow should change. Or open
				<strong class="text-foreground">Learn</strong>, hold Right Alt (L2 stand-in), and confirm the
				board shows Adjust — mostly ▽ on the right half.
			</p>
		{:else if !last}
			<div class="font-mono text-2xl tracking-tight text-muted-foreground md:text-3xl">
				Waiting for HID…
			</div>
			<p class="mt-3 max-w-2xl text-sm text-muted-foreground">
				Focus this page and press keys on the Lily58.
			</p>
		{:else}
			<div class="font-mono text-3xl tracking-tight md:text-4xl">
				{last.label ?? 'unmapped'}
				<span class="text-muted-foreground"> · {last.type}</span>
			</div>
			<p class="mt-2 font-mono text-base text-muted-foreground md:text-lg">
				<code class="text-foreground">{last.code}</code>
				<span class="mx-2 opacity-40">·</span>
				key <code class="text-foreground">{last.key === ' ' ? 'Space' : last.key}</code>
				{#if last.layerId}
					<span class="mx-2 opacity-40">·</span>
					<span class="text-foreground">{last.layerId}</span>
				{/if}
				{#if last.mappedIndex !== null}
					<span class="mx-2 opacity-40">·</span>
					#{last.mappedIndex}
				{/if}
			</p>
		{/if}
	</div>

	<div class="flex min-h-14 flex-wrap items-center gap-4 rounded-lg bg-[#1c1c1c] px-4 py-3">
		{#if suspect}
			<span class="font-mono text-[0.65rem] uppercase tracking-widest text-foreground">Suspect</span>
			<span class="font-mono text-xl">
				{testState.suspectLabel}
				<span class="text-muted-foreground">
					#{suspect.index} · row {suspect.row}, col {suspect.col}
					{#if isMo}
						· &amp;mo → {targetId}
					{/if}
				</span>
			</span>
			<span class="max-w-xl text-sm text-muted-foreground">{diagnosisCopy}</span>
		{:else}
			<span class="font-mono text-sm text-muted-foreground">
				Click the dead key on the board, or hit Right L2.
			</span>
		{/if}
	</div>
</section>
