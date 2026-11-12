<script lang="ts">
	import BuildFlash from '#lib/components/firmware/BuildFlash.svelte';
	import StudioEditor from '#lib/components/studio/StudioEditor.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { firmwareState } from '#lib/state/firmware.svelte';
</script>

<section class="space-y-6" aria-label="Firmware hub">
	<div class="flex flex-wrap gap-1" role="tablist" aria-label="Firmware panels">
		<Button
			type="button"
			size="sm"
			variant={firmwareState.panel === 'live' ? 'default' : 'ghost'}
			onclick={() => firmwareState.setPanel('live')}
		>
			Live
		</Button>
		<Button
			type="button"
			size="sm"
			variant={firmwareState.panel === 'build' ? 'default' : 'ghost'}
			onclick={() => firmwareState.setPanel('build')}
		>
			Build / Flash
		</Button>
	</div>

	{#if firmwareState.panel === 'live'}
		<StudioEditor />
	{:else}
		<BuildFlash />
	{/if}
</section>
