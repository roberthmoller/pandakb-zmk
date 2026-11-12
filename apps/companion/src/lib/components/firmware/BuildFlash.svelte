<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { firmwareState } from '#lib/state/firmware.svelte';

	async function probe() {
		firmwareState.persistWestRoot();
		const res = await fetch(
			`/api/firmware/config?root=${encodeURIComponent(firmwareState.westRoot)}`
		);
		const data = await res.json();
		if (!res.ok) {
			firmwareState.flashStatus = data.message ?? 'Config probe failed';
			return;
		}
		firmwareState.artifacts = {
			left: data.artifacts.left ?? undefined,
			right: data.artifacts.right ?? undefined
		};
		firmwareState.flashStatus = data.bootloaderMounted
			? 'Bootloader volume mounted (/Volumes/RPI-RP2)'
			: 'Bootloader not mounted — double-reset a half when ready to flash';
	}

	async function build() {
		firmwareState.persistWestRoot();
		firmwareState.buildStatus = 'running';
		firmwareState.buildLog = '';
		const res = await fetch('/api/firmware/build', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ root: firmwareState.westRoot })
		});
		if (!res.ok || !res.body) {
			firmwareState.buildStatus = 'error';
			firmwareState.buildLog = `HTTP ${res.status}`;
			return;
		}
		const reader = res.body.getReader();
		const decoder = new TextDecoder();
		let full = '';
		while (true) {
			const { done, value } = await reader.read();
			if (done) break;
			full += decoder.decode(value, { stream: true });
			firmwareState.buildLog = full;
		}
		const ok = full.includes('\nOK\n');
		firmwareState.buildStatus = ok ? 'ok' : 'error';
		const left = /LEFT_UF2=(.+)/.exec(full)?.[1]?.trim();
		const right = /RIGHT_UF2=(.+)/.exec(full)?.[1]?.trim();
		if (left || right) {
			firmwareState.artifacts = { left, right };
		}
	}

	async function flash(half: 'left' | 'right') {
		firmwareState.flashStatus = `Flashing ${half}…`;
		const res = await fetch('/api/firmware/flash', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ root: firmwareState.westRoot, half })
		});
		const data = await res.json().catch(() => ({}));
		if (!res.ok) {
			firmwareState.flashStatus = data.message ?? `Flash ${half} failed (${res.status})`;
			return;
		}
		firmwareState.flashStatus = `Flashed ${half} → ${data.to}`;
	}
</script>

<section class="space-y-4" aria-label="Build and flash">
	<div>
		<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
			Local west
		</div>
		<p class="mt-1 max-w-2xl text-sm text-muted-foreground">
			Use this when Live remap has no free layer slots, or you need a Studio / west config change.
			Point at your <code class="text-foreground">pandakb-lily</code> west tree, build both halves,
			then flash each UF2 after a double-reset.
		</p>
	</div>

	<label class="block max-w-3xl space-y-1 text-sm">
		<span class="text-muted-foreground">WEST_ROOT</span>
		<input
			class="h-10 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-3 font-mono text-xs"
			bind:value={firmwareState.westRoot}
			onchange={() => firmwareState.persistWestRoot()}
		/>
	</label>

	<div class="flex flex-wrap gap-2">
		<Button size="sm" variant="outline" onclick={probe}>Probe artifacts</Button>
		<Button
			size="sm"
			onclick={build}
			disabled={firmwareState.buildStatus === 'running'}
		>
			{firmwareState.buildStatus === 'running' ? 'Building…' : 'Build left + right'}
		</Button>
		<Button
			size="sm"
			variant="outline"
			onclick={() => flash('left')}
			disabled={!firmwareState.artifacts.left}
		>
			Flash left
		</Button>
		<Button
			size="sm"
			variant="outline"
			onclick={() => flash('right')}
			disabled={!firmwareState.artifacts.right}
		>
			Flash right
		</Button>
	</div>

	{#if firmwareState.flashStatus}
		<p class="font-mono text-sm text-muted-foreground">{firmwareState.flashStatus}</p>
	{/if}

	{#if firmwareState.artifacts.left || firmwareState.artifacts.right}
		<div class="font-mono text-xs text-muted-foreground">
			{#if firmwareState.artifacts.left}
				<div>L {firmwareState.artifacts.left}</div>
			{/if}
			{#if firmwareState.artifacts.right}
				<div>R {firmwareState.artifacts.right}</div>
			{/if}
		</div>
	{/if}

	{#if firmwareState.buildLog}
		<pre
			class="max-h-80 overflow-auto rounded-lg bg-[#0a0a0a] p-4 font-mono text-[0.65rem] leading-relaxed text-muted-foreground"
		>{firmwareState.buildLog}</pre>
	{/if}
</section>
