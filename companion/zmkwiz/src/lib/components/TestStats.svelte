<script lang="ts">
	import { lily58 } from '#lib/boards/lily58';
	import { testState } from '#lib/state/test.svelte';

	const seenCount = $derived(testState.seen.length);
	const totalKeys = lily58.keys.length;
	const diagnosis = $derived(testState.diagnosis);
	const suspect = $derived(testState.suspect);
	const isMo = $derived(testState.suspectIsMomentary);

	const steps = $derived.by(() => {
		if (!suspect) {
			return [
				'Mark Right L2 (Raise) on the board',
				'Hold L2 and tap J — expect ←, not j',
				'If that fails, continuity-check the L2 switch with power off'
			];
		}
		switch (diagnosis) {
			case 'mo-probe':
				return isMo && testState.suspectTargetLayerId === 'adjust'
					? [
							'L2+J → j is normal on stock PandaKB (Adjust is &trans there)',
							'Hold L2 + W/S to tweak RGB — proves the switch if underglow moves',
							'Rebuild with the patched lily58.keymap (Raise inserted) for arrows on L2+J',
							'Multimeter only if RGB probe also fails'
						]
					: [
							'Focus this page with the Lily58 connected',
							'Hold right L2 and tap a Raise-only key (e.g. J → ←)',
							'Only open the multimeter if that still fails'
						];
			case 'mo-ok':
				return [
					'Firmware path is good — L2 is closing the matrix',
					'No multimeter needed unless the feel is mechanically bad'
				];
			case 'mo-dead':
				return [
					'Power off and disconnect batteries before probing',
					`Continuity on L2 switch pads (row ${suspect.row}, col ${suspect.col}) while held — should beep`,
					'Diode mode across the key’s diode; compare to Enter on the same half',
					'If switch + diode are good: reflow pads, then buzz col 9 / row 4'
				];
			case 'working':
				return [
					'Key is electrically fine from the host’s view',
					'Re-check keymap / layer if the character is wrong'
				];
			case 'row':
				return [
					`Continuity: row ${suspect.row} path on this half`,
					'Inspect diodes along that row before replacing switches'
				];
			case 'col':
				return [
					`Continuity: col ${suspect.col} path on this half`,
					'On a split, also check the interconnect if a whole column died'
				];
			default:
				return [
					'Power off before probing',
					'Continuity across the two switch pins while held',
					'Check the diode, then reflow pads if both measure fine'
				];
		}
	});
</script>

<section
	class="flex flex-wrap items-start justify-between gap-6 border-t border-white/10 pt-4"
	aria-label="Test details"
>
	<div class="flex flex-wrap gap-8">
		<div>
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">Seen</div>
			<div class="text-3xl font-semibold tracking-tight">{seenCount}/{totalKeys}</div>
		</div>
		<div>
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				Downs
			</div>
			<div class="text-3xl font-semibold tracking-tight">{testState.totalDowns}</div>
		</div>
		{#if isMo}
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Raise probes
				</div>
				<div class="text-3xl font-semibold tracking-tight">{testState.layerProbeHits}</div>
			</div>
		{:else}
			<div>
				<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
					Unmapped
				</div>
				<div class="text-3xl font-semibold tracking-tight">{testState.unmappedCount}</div>
			</div>
		{/if}
	</div>

	<div class="min-w-[16rem] flex-1 space-y-2">
		<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
			{#if isMo && diagnosis !== 'mo-ok'}
				Multimeter — right L2
			{:else}
				Next steps
			{/if}
		</div>
		<ol class="space-y-1.5 text-sm text-muted-foreground">
			{#each steps as step, i (step)}
				<li class="flex gap-2">
					<span class="font-mono text-foreground">{i + 1}.</span>
					<span>{step}</span>
				</li>
			{/each}
		</ol>
	</div>

	{#if testState.log.length > 0}
		<div class="w-full max-w-md font-mono text-[0.7rem] text-muted-foreground md:w-auto">
			<div class="mb-1 uppercase tracking-widest">Recent HID</div>
			<ul class="max-h-28 space-y-0.5 overflow-y-auto">
				{#each testState.log.slice(0, 8) as entry (entry.at + entry.type + entry.code)}
					<li>
						<span class="text-foreground">{entry.type}</span>
						{entry.code}
						{#if entry.label}
							→ {entry.label}
						{:else}
							→ ?
						{/if}
						{#if entry.impliesLayer && entry.layerId}
							<span class="text-foreground">({entry.layerId})</span>
						{/if}
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</section>
