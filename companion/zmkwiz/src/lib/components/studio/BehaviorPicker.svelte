<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import type { BehaviorBinding } from '@zmkfirmware/zmk-studio-ts-client/keymap';
	import { liveKeymap } from '#lib/studio/liveKeymap.svelte';
	import { hidUsageFromPageAndId, hidUsageLabel, keyboardUsagePages } from '#lib/studio/hid';

	type Props = {
		binding: BehaviorBinding;
		onapply: (binding: BehaviorBinding) => void;
	};

	let { binding, onapply }: Props = $props();

	let behaviorId = $state('0');
	let param1 = $state(0);
	let param2 = $state(0);

	$effect(() => {
		behaviorId = String(binding.behaviorId);
		param1 = binding.param1;
		param2 = binding.param2;
	});

	const behaviorIdNum = $derived(Number(behaviorId));
	const details = $derived(liveKeymap.behaviors[behaviorIdNum]);
	const needsHid = $derived(
		Boolean(
			details?.metadata?.some((set) => set.param1?.some((p) => p.hidUsage !== undefined))
		)
	);
	const needsLayer = $derived(
		Boolean(details?.metadata?.some((set) => set.param1?.some((p) => p.layerId !== undefined)))
	);

	const pages = keyboardUsagePages();

	function apply() {
		onapply({ behaviorId: behaviorIdNum, param1: param1 || 0, param2: param2 || 0 });
	}
</script>

<div class="space-y-3 rounded-lg bg-[#1c1c1c] px-4 py-4">
	<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
		Binding
	</div>

	<label class="block space-y-1 text-sm">
		<span class="text-muted-foreground">Behavior</span>
		<select
			class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
			bind:value={behaviorId}
			onchange={() => {
				param1 = 0;
				param2 = 0;
			}}
		>
			{#each liveKeymap.behaviorList as b (b.id)}
				<option value={String(b.id)}>{b.displayName}</option>
			{/each}
		</select>
	</label>

	{#if needsHid}
		<label class="block space-y-1 text-sm">
			<span class="text-muted-foreground">Key (HID)</span>
			<select
				class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
				value={param1}
				onchange={(e) => {
					param1 = Number((e.currentTarget as HTMLSelectElement).value);
				}}
			>
				<option value={0}>—</option>
				{#each pages as page (page.Id)}
					{#each page.UsageIds ?? [] as usage (usage.Id)}
						<option value={hidUsageFromPageAndId(page.Id, usage.Id)}>
							{hidUsageLabel(hidUsageFromPageAndId(page.Id, usage.Id))}
						</option>
					{/each}
				{/each}
			</select>
		</label>
	{:else if needsLayer}
		<label class="block space-y-1 text-sm">
			<span class="text-muted-foreground">Layer</span>
			<select
				class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
				value={String(param1)}
				onchange={(e) => {
					param1 = Number((e.currentTarget as HTMLSelectElement).value);
				}}
			>
				{#each liveKeymap.keymap?.layers ?? [] as layer, i (layer.id)}
					<option value={String(layer.id)}
						>{liveKeymap.layerLabel(layer, i)} (id {layer.id})</option
					>
				{/each}
			</select>
		</label>
	{:else}
		<div class="grid grid-cols-2 gap-2">
			<label class="block space-y-1 text-sm">
				<span class="text-muted-foreground">Param 1</span>
				<input
					type="number"
					class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
					bind:value={param1}
				/>
			</label>
			<label class="block space-y-1 text-sm">
				<span class="text-muted-foreground">Param 2</span>
				<input
					type="number"
					class="h-9 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-2 font-mono text-sm"
					bind:value={param2}
				/>
			</label>
		</div>
	{/if}

	<Button size="sm" onclick={apply} disabled={liveKeymap.busy}>Apply to key</Button>
</div>
