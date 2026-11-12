<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { studioConnection } from '#lib/studio/connection.svelte';
	import { liveKeymap } from '#lib/studio/liveKeymap.svelte';
	import { LockState } from '@zmkfirmware/zmk-studio-ts-client/core';

	async function connect() {
		await studioConnection.connect();
		if (studioConnection.status === 'connected') {
			await liveKeymap.refresh();
		}
	}

	function disconnect() {
		studioConnection.disconnect();
		liveKeymap.clear();
	}

	const lockLabel = $derived(
		studioConnection.lockState === LockState.ZMK_STUDIO_CORE_LOCK_STATE_UNLOCKED
			? 'unlocked'
			: 'locked'
	);
</script>

<div class="flex flex-wrap items-center gap-3 rounded-lg bg-[#171717] px-4 py-3">
	{#if studioConnection.status === 'connected'}
		<Badge variant="secondary">{studioConnection.deviceName}</Badge>
		<span class="font-mono text-xs uppercase tracking-widest text-muted-foreground">
			{lockLabel}
			{#if liveKeymap.unsaved}
				· unsaved
			{/if}
		</span>
		<Button size="sm" variant="outline" onclick={() => liveKeymap.refresh()} disabled={liveKeymap.busy}>
			Refresh
		</Button>
		<Button size="sm" variant="outline" onclick={disconnect}>Disconnect</Button>
	{:else}
		<span class="text-sm text-muted-foreground">
			Plug the <strong class="text-foreground">left/central</strong> half over USB (data cable).
		</span>
		<Button
			size="sm"
			onclick={connect}
			disabled={studioConnection.status === 'connecting'}
		>
			{studioConnection.status === 'connecting' ? 'Connecting…' : 'Connect USB'}
		</Button>
	{/if}
	{#if studioConnection.error}
		<span class="w-full text-sm text-destructive">{studioConnection.error}</span>
	{/if}
</div>
