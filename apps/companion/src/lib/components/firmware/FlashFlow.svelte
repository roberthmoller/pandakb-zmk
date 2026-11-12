<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		buildFirmware,
		flashHalf,
		getFirmwareStatus,
		isTauri,
		onFirmwareDone,
		onFirmwareLog,
		waitBootloader,
		westUpdate
	} from '#lib/firmware/tauri';
	import { firmwareState } from '#lib/state/firmware.svelte';
	import { onMount } from 'svelte';

	let logEl = $state<HTMLPreElement | null>(null);

	$effect(() => {
		firmwareState.buildLog;
		if (logEl) logEl.scrollTop = logEl.scrollHeight;
	});

	async function refresh() {
		firmwareState.persistWestRoot();
		if (!isTauri()) {
			firmwareState.flashStatus = 'Open this window inside the Tauri app (npm run tauri:dev).';
			return;
		}
		try {
			firmwareState.status = await getFirmwareStatus(firmwareState.westRoot);
			if (firmwareState.status.westRoot && firmwareState.status.westRoot !== firmwareState.westRoot) {
				firmwareState.westRoot = firmwareState.status.westRoot;
			}
		} catch (e) {
			firmwareState.flashStatus = String(e);
		}
	}

	async function runBuild() {
		if (!isTauri()) return;
		firmwareState.persistWestRoot();
		firmwareState.resetLog();
		firmwareState.buildStatus = 'running';
		firmwareState.flashStatus = null;
		firmwareState.step = 'build';
		try {
			await buildFirmware(firmwareState.westRoot, firmwareState.updateFirst);
			firmwareState.buildStatus = 'ok';
			firmwareState.step = 'flash-left';
			await refresh();
		} catch (e) {
			firmwareState.buildStatus = 'error';
			firmwareState.appendLog(String(e));
			firmwareState.flashStatus = String(e);
		}
	}

	async function runUpdateOnly() {
		if (!isTauri()) return;
		firmwareState.persistWestRoot();
		firmwareState.resetLog();
		firmwareState.buildStatus = 'running';
		try {
			await westUpdate(firmwareState.westRoot);
			firmwareState.buildStatus = 'ok';
			await refresh();
		} catch (e) {
			firmwareState.buildStatus = 'error';
			firmwareState.appendLog(String(e));
			firmwareState.flashStatus = String(e);
		}
	}

	async function flash(half: 'left' | 'right') {
		if (!isTauri()) return;
		firmwareState.flashStatus = null;
		firmwareState.waitingBootloader = true;
		firmwareState.flashStatus = `Waiting for /Volumes/RPI-RP2 — double-reset the ${half} nice!nano…`;
		try {
			const mounted = await waitBootloader(120);
			if (!mounted) {
				firmwareState.flashStatus = 'Timed out waiting for bootloader volume.';
				firmwareState.waitingBootloader = false;
				return;
			}
			firmwareState.flashStatus = `Flashing ${half}…`;
			const result = await flashHalf(half, firmwareState.westRoot);
			firmwareState.flashStatus = `Flashed ${half} → ${result.to}`;
			if (half === 'left') firmwareState.step = 'flash-right';
			else firmwareState.step = 'done';
			await refresh();
		} catch (e) {
			firmwareState.flashStatus = String(e);
		} finally {
			firmwareState.waitingBootloader = false;
		}
	}

	onMount(() => {
		let unsubs: Array<() => void> = [];
		(async () => {
			await refresh();
			if (!isTauri()) return;
			unsubs.push(await onFirmwareLog((line) => firmwareState.appendLog(line)));
			unsubs.push(
				await onFirmwareDone(() => {
					/* status refreshed by caller */
				})
			);
		})();
		const id = setInterval(() => {
			if (isTauri() && !firmwareState.status?.busy) refresh();
		}, 4000);
		return () => {
			clearInterval(id);
			unsubs.forEach((u) => u());
		};
	});

	const s = $derived(firmwareState.status);
	const busy = $derived(firmwareState.buildStatus === 'running' || firmwareState.waitingBootloader);
</script>

<section class="space-y-8" aria-label="Flash companion firmware">
	<header class="space-y-2">
		<p class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
			Companion OLED pair
		</p>
		<h1 class="font-heading text-3xl font-semibold tracking-tight md:text-4xl">Build &amp; flash</h1>
		<p class="max-w-2xl text-sm text-muted-foreground">
			Builds left (<code class="text-foreground">Lily58_L nice_oled raw_hid_adapter app_layer</code>)
			and right (<code class="text-foreground">Lily58_R game_badge</code>) via local west (or Docker),
			then copies each UF2 onto the nice!nano bootloader volume.
		</p>
	</header>

	<ol class="grid gap-2 sm:grid-cols-5">
		{#each [
			{ id: 'setup', label: 'Setup' },
			{ id: 'build', label: 'Build' },
			{ id: 'flash-left', label: 'Flash L' },
			{ id: 'flash-right', label: 'Flash R' },
			{ id: 'done', label: 'Done' }
		] as item}
			<li
				class="rounded-md border px-3 py-2 font-mono text-[0.65rem] uppercase tracking-widest
				{firmwareState.step === item.id
					? 'border-white/30 bg-white/10 text-foreground'
					: 'border-white/10 text-muted-foreground'}"
			>
				{item.label}
			</li>
		{/each}
	</ol>

	<div class="space-y-3 rounded-lg border border-white/10 p-4">
		<label class="block space-y-1 text-sm">
			<span class="text-muted-foreground">West workspace</span>
			<input
				class="h-10 w-full rounded-md border border-white/10 bg-[#0a0a0a] px-3 font-mono text-xs"
				bind:value={firmwareState.westRoot}
				onchange={() => refresh()}
				disabled={busy}
			/>
		</label>

		<div class="flex flex-wrap gap-3 text-xs text-muted-foreground">
			<span>
				Runner:
				{#if s?.westBin}
					<span class="text-foreground">west</span>
				{:else if s?.dockerOk}
					<span class="text-foreground">docker ({`zmk-build-arm`})</span>
				{:else}
					<span class="text-foreground">none</span>
				{/if}
			</span>
			<span>
				Workspace:
				<span class={s?.westRootOk ? 'text-foreground' : 'text-foreground'}>
					{s?.westRootOk ? 'ok' : 'needs init/update'}
				</span>
			</span>
			<span>
				Bootloader:
				<span class="text-foreground">{s?.bootloaderMounted ? 'mounted' : 'not mounted'}</span>
			</span>
		</div>

		{#if s?.repoRoot}
			<p class="font-mono text-[0.65rem] text-muted-foreground">Repo sync from {s.repoRoot}</p>
		{/if}

		<label class="flex items-center gap-2 text-sm text-muted-foreground">
			<input type="checkbox" bind:checked={firmwareState.updateFirst} disabled={busy} />
			Run <code class="text-foreground">west update</code> before build
		</label>

		<div class="flex flex-wrap gap-2">
			<Button size="sm" variant="outline" onclick={refresh} disabled={busy}>Probe</Button>
			<Button size="sm" variant="outline" onclick={runUpdateOnly} disabled={busy}>
				West update
			</Button>
			<Button size="sm" onclick={runBuild} disabled={busy}>
				{firmwareState.buildStatus === 'running' ? 'Building…' : 'Build left + right'}
			</Button>
		</div>
	</div>

	<div class="space-y-3 rounded-lg border border-white/10 p-4">
		<p class="text-sm text-muted-foreground">
			Double-reset a half so it mounts as <code class="text-foreground">/Volumes/RPI-RP2</code>, then
			flash. Do <strong class="font-medium text-foreground">right</strong> first if you prefer, but the
			guided order is left → right. Leave the <strong class="font-medium text-foreground">left</strong> half
			plugged in over USB afterwards for Raw HID.
		</p>
		<div class="flex flex-wrap gap-2">
			<Button
				size="sm"
				variant="outline"
				onclick={() => flash('left')}
				disabled={busy || (!s?.leftUf2 && firmwareState.buildStatus !== 'ok')}
			>
				{firmwareState.waitingBootloader && firmwareState.step === 'flash-left'
					? 'Waiting for left…'
					: 'Flash left'}
			</Button>
			<Button
				size="sm"
				variant="outline"
				onclick={() => flash('right')}
				disabled={busy || (!s?.rightUf2 && firmwareState.buildStatus !== 'ok')}
			>
				{firmwareState.waitingBootloader && firmwareState.step === 'flash-right'
					? 'Waiting for right…'
					: 'Flash right'}
			</Button>
		</div>
		{#if s?.leftUf2 || s?.rightUf2}
			<div class="font-mono text-[0.65rem] text-muted-foreground">
				{#if s?.leftUf2}<div>L {s.leftUf2}</div>{/if}
				{#if s?.rightUf2}<div>R {s.rightUf2}</div>{/if}
			</div>
		{/if}
		{#if firmwareState.flashStatus}
			<p class="font-mono text-sm text-muted-foreground">{firmwareState.flashStatus}</p>
		{/if}
		{#if firmwareState.step === 'done'}
			<p class="text-sm text-foreground">Both halves flashed. Plug the left half over USB and keep the tray companion running.</p>
		{/if}
	</div>

	{#if firmwareState.buildLog}
		<pre
			bind:this={logEl}
			class="max-h-96 overflow-auto rounded-lg bg-[#0a0a0a] p-4 font-mono text-[0.65rem] leading-relaxed text-muted-foreground"
		>{firmwareState.buildLog}</pre>
	{/if}
</section>
