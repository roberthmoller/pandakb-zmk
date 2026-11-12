<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import {
		buildFirmware,
		enterBootloader,
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
			firmwareState.message = 'Open PandaKB from the menu bar app to flash.';
			return;
		}
		try {
			firmwareState.status = await getFirmwareStatus(firmwareState.westRoot);
			if (
				firmwareState.status.westRoot &&
				firmwareState.status.westRoot !== firmwareState.westRoot
			) {
				firmwareState.westRoot = firmwareState.status.westRoot;
			}
		} catch (e) {
			firmwareState.message = String(e);
		}
	}

	async function softEnterThenFlash(half: 'left' | 'right') {
		firmwareState.wizardHalf = half;
		firmwareState.appendLog(
			half === 'left'
				? 'Left half: trying soft-enter bootloader over Raw HID…'
				: 'Right half: trying soft-enter bootloader (may need a cable)…'
		);

		let softOk = false;
		try {
			await enterBootloader(half);
			softOk = true;
			firmwareState.appendLog(`Soft-enter ${half} requested.`);
		} catch (e) {
			/* Soft-enter fails on stock / pre-0x30 firmware — fall through to manual wizard. */
			firmwareState.appendLog(
				`Soft-enter ${half} failed (${String(e)}). Manual bootloader needed.`
			);
		}

		const mounted = await waitBootloader(softOk ? 20 : 8);
		if (!mounted) {
			firmwareState.wizardHalf = half;
			firmwareState.wizard = 'bootloader';
			firmwareState.phase = 'wizard-bootloader';
			firmwareState.appendLog(
				half === 'left'
					? 'Left NICENANO not mounted — show left bootloader wizard.'
					: 'Right NICENANO not mounted — show right bootloader wizard.'
			);
			throw new Error('WIZARD_BOOTLOADER');
		}

		firmwareState.appendLog(`Bootloader mounted — flashing ${half}…`);
		const result = await flashHalf(half, firmwareState.westRoot);
		firmwareState.message = null;
		firmwareState.appendLog(`Flashed ${half}: ${result.from} → ${result.to}`);
		await new Promise((r) => setTimeout(r, 1200));
	}

	async function runFlash() {
		if (!isTauri() || firmwareState.running) return;
		firmwareState.running = true;
		firmwareState.resetLog();
		firmwareState.message = null;
		firmwareState.clearWizard();
		firmwareState.showLog = false;

		try {
			firmwareState.persistWestRoot();
			await refresh();
			const haveArtifacts = Boolean(
				firmwareState.status?.leftUf2 && firmwareState.status?.rightUf2
			);
			const mustBuild =
				!haveArtifacts || firmwareState.forceBuild || firmwareState.updateFirst;

			if (mustBuild) {
				firmwareState.phase = 'building';
				await buildFirmware(firmwareState.westRoot, {
					updateFirst: firmwareState.updateFirst,
					force: firmwareState.forceBuild
				});
				await refresh();
			} else {
				firmwareState.appendLog(
					`Flashing repo artifacts (skip west rebuild):\n  L ${firmwareState.status?.leftUf2}\n  R ${firmwareState.status?.rightUf2}`
				);
			}

			firmwareState.phase = 'flash-left';
			await softEnterThenFlash('left');

			firmwareState.phase = 'flash-right';
			try {
				await softEnterThenFlash('right');
			} catch (e) {
				if (String(e).includes('WIZARD_BOOTLOADER')) {
					firmwareState.running = false;
					return;
				}
				firmwareState.wizard = 'right-cable';
				firmwareState.wizardHalf = 'right';
				firmwareState.phase = 'wizard-right';
				firmwareState.running = false;
				return;
			}

			firmwareState.phase = 'done';
			await refresh();
		} catch (e) {
			if (String(e).includes('WIZARD_BOOTLOADER')) {
				firmwareState.running = false;
				return;
			}
			firmwareState.phase = 'error';
			firmwareState.appendLog(String(e));
			firmwareState.message = friendlyError(String(e));
		} finally {
			if (
				firmwareState.phase !== 'wizard-right' &&
				firmwareState.phase !== 'wizard-bootloader'
			) {
				firmwareState.running = false;
			}
		}
	}

	async function wizardContinue() {
		if (!isTauri() || firmwareState.running) return;
		const half = firmwareState.wizardHalf;
		firmwareState.running = true;
		firmwareState.clearWizard();
		firmwareState.phase = half === 'left' ? 'flash-left' : 'flash-right';

		try {
			const mounted = await waitBootloader(5);
			if (!mounted) {
				try {
					await enterBootloader(half);
				} catch {
					/* ignore */
				}
			}
			const ready = await waitBootloader(90);
			if (!ready) {
				firmwareState.wizard = 'bootloader';
				firmwareState.phase = 'wizard-bootloader';
				firmwareState.running = false;
				return;
			}
			await flashHalf(half, firmwareState.westRoot);
			await new Promise((r) => setTimeout(r, 1200));

			if (half === 'left') {
				firmwareState.phase = 'flash-right';
				try {
					await softEnterThenFlash('right');
				} catch (e) {
					if (String(e).includes('WIZARD_BOOTLOADER')) {
						firmwareState.running = false;
						return;
					}
					firmwareState.wizard = 'right-cable';
					firmwareState.wizardHalf = 'right';
					firmwareState.phase = 'wizard-right';
					firmwareState.running = false;
					return;
				}
			}

			firmwareState.phase = 'done';
			await refresh();
		} catch (e) {
			if (String(e).includes('WIZARD_BOOTLOADER')) {
				firmwareState.running = false;
				return;
			}
			firmwareState.phase = 'error';
			firmwareState.message = friendlyError(String(e));
		} finally {
			if (
				firmwareState.phase !== 'wizard-right' &&
				firmwareState.phase !== 'wizard-bootloader'
			) {
				firmwareState.running = false;
			}
		}
	}

	function cancelWizard() {
		firmwareState.clearWizard();
		firmwareState.phase = 'idle';
		firmwareState.running = false;
		firmwareState.message = null;
	}

	async function runUpdateOnly() {
		if (!isTauri() || firmwareState.running) return;
		firmwareState.running = true;
		firmwareState.resetLog();
		firmwareState.phase = 'building';
		firmwareState.showLog = true;
		try {
			await westUpdate(firmwareState.westRoot);
			firmwareState.phase = 'idle';
			await refresh();
		} catch (e) {
			firmwareState.phase = 'error';
			firmwareState.message = friendlyError(String(e));
			firmwareState.appendLog(String(e));
		} finally {
			firmwareState.running = false;
		}
	}

	function friendlyError(raw: string) {
		if (raw.includes('west') || raw.includes('docker')) {
			return 'Could not build firmware. Check Advanced for the west workspace.';
		}
		if (raw.includes('Bootloader') || raw.includes('NICENANO')) {
			return 'Waiting for the keyboard bootloader. Double-tap reset on that half.';
		}
		return raw.length > 160 ? raw.slice(0, 160) + '…' : raw;
	}

	onMount(() => {
		const unsubs: Array<() => void> = [];
		(async () => {
			await refresh();
			if (!isTauri()) return;
			unsubs.push(await onFirmwareLog((line) => firmwareState.appendLog(line)));
			unsubs.push(await onFirmwareDone(() => {}));
		})();
		const id = setInterval(() => {
			if (isTauri() && !firmwareState.running) refresh();
		}, 5000);
		return () => {
			clearInterval(id);
			unsubs.forEach((u) => u());
		};
	});

	const busy = $derived(firmwareState.running);
	const inWizard = $derived(
		firmwareState.phase === 'wizard-right' || firmwareState.phase === 'wizard-bootloader'
	);
	const progress = $derived(
		(
			{
				idle: 0,
				building: 25,
				'flash-left': 55,
				'flash-right': 85,
				'wizard-right': 70,
				'wizard-bootloader': 50,
				done: 100,
				error: 0
			} as const
		)[firmwareState.phase]
	);
	const statusLine = $derived(
		(
			{
				idle: 'Ready when you are',
				building: 'Preparing firmware…',
				'flash-left': 'Updating left half…',
				'flash-right': 'Updating right half…',
				'wizard-right': 'One more step',
				'wizard-bootloader': 'One more step',
				done: 'Keyboard updated',
				error: 'Something went wrong'
			} as const
		)[firmwareState.phase]
	);
</script>

<section
	class="flash-flow flex min-h-[min(70dvh,640px)] flex-col justify-center"
	aria-label="Flash keyboard"
>
	{#if inWizard}
		<!-- Guided step — only when automation needs the user -->
		<div class="mx-auto w-full max-w-md space-y-8 text-center">
			<p class="eyebrow">
				{#if firmwareState.wizardHalf === 'left'}
					Left half · USB
				{:else}
					Right half
				{/if}
			</p>
			{#if firmwareState.wizard === 'right-cable'}
				<h1 class="font-heading text-3xl font-semibold tracking-tight text-foreground">
					Plug in the right half
				</h1>
				<p class="mx-auto max-w-sm text-sm leading-relaxed text-muted-foreground">
					Use a second cable, or move the USB cable to the right nice!nano. Both halves can stay
					connected if you have a hub.
				</p>
				<ol
					class="mx-auto max-w-sm space-y-2 text-left text-sm leading-relaxed text-muted-foreground"
				>
					<li>
						<span class="font-medium text-foreground">1.</span> Plug USB into the
						<strong class="font-medium text-foreground"> right </strong> nice!nano.
					</li>
					<li>
						<span class="font-medium text-foreground">2.</span> Double-tap its reset button so a
						<strong class="font-medium text-foreground"> NICENANO </strong> drive appears.
					</li>
					<li>
						<span class="font-medium text-foreground">3.</span> Press Continue.
					</li>
				</ol>
			{:else if firmwareState.wizardHalf === 'left'}
				<h1 class="font-heading text-3xl font-semibold tracking-tight text-foreground">
					Put the left half in bootloader
				</h1>
				<p class="mx-auto max-w-sm text-sm leading-relaxed text-muted-foreground">
					Soft-enter didn’t work, so the left half needs a manual reset. This is the half with the
					USB cable (central / master).
				</p>
				<ol
					class="mx-auto max-w-sm space-y-2 text-left text-sm leading-relaxed text-muted-foreground"
				>
					<li>
						<span class="font-medium text-foreground">1.</span> Keep USB plugged into the
						<strong class="font-medium text-foreground"> left </strong> nice!nano.
					</li>
					<li>
						<span class="font-medium text-foreground">2.</span> Quickly
						<strong class="font-medium text-foreground"> double-tap the reset button </strong>
						on that board (small button on the controller).
					</li>
					<li>
						<span class="font-medium text-foreground">3.</span> Wait until a
						<strong class="font-medium text-foreground"> NICENANO </strong> drive shows up in
						Finder (sometimes named RPI-RP2).
					</li>
					<li>
						<span class="font-medium text-foreground">4.</span> Press Continue — we’ll copy the
						left UF2 onto that drive.
					</li>
				</ol>
			{:else}
				<h1 class="font-heading text-3xl font-semibold tracking-tight text-foreground">
					Put the right half in bootloader
				</h1>
				<p class="mx-auto max-w-sm text-sm leading-relaxed text-muted-foreground">
					Soft-enter didn’t reach the right half. Plug it in (or use a hub), then reset it
					manually.
				</p>
				<ol
					class="mx-auto max-w-sm space-y-2 text-left text-sm leading-relaxed text-muted-foreground"
				>
					<li>
						<span class="font-medium text-foreground">1.</span> Plug USB into the
						<strong class="font-medium text-foreground"> right </strong> nice!nano.
					</li>
					<li>
						<span class="font-medium text-foreground">2.</span> Quickly
						<strong class="font-medium text-foreground"> double-tap reset </strong>
						until
						<strong class="font-medium text-foreground"> NICENANO </strong> appears.
					</li>
					<li>
						<span class="font-medium text-foreground">3.</span> Press Continue.
					</li>
				</ol>
			{/if}

			<div class="flex flex-col items-stretch gap-3 pt-2 sm:flex-row sm:justify-center">
				<Button size="lg" onclick={wizardContinue} disabled={busy}>
					{busy
						? 'Working…'
						: firmwareState.wizardHalf === 'left'
							? 'Continue · flash left'
							: 'Continue · flash right'}
				</Button>
				<Button size="lg" variant="ghost" onclick={cancelWizard} disabled={busy}>Cancel</Button>
			</div>
		</div>
	{:else if firmwareState.phase === 'done'}
		<div class="mx-auto w-full max-w-md space-y-8 text-center">
			<p class="eyebrow">PandaKB</p>
			<h1 class="font-heading text-3xl font-semibold tracking-tight text-foreground md:text-4xl">
				You're set
			</h1>
			<p class="mx-auto max-w-sm text-sm leading-relaxed text-muted-foreground">
				Both halves are updated. Keep the left half on USB so the companion can talk to the
				keyboard.
			</p>
			<Button size="lg" variant="outline" onclick={() => (firmwareState.phase = 'idle')}>
				Done
			</Button>
		</div>
	{:else}
		<!-- Default: one quiet action -->
		<div class="mx-auto w-full max-w-md space-y-10 text-center">
			<div class="space-y-3">
				<p class="eyebrow">PandaKB</p>
				<h1 class="font-heading text-4xl font-semibold tracking-tight text-foreground md:text-5xl">
					Flash
				</h1>
				<p class="mx-auto max-w-xs text-sm leading-relaxed text-muted-foreground">
					{#if busy}
						{statusLine}
					{:else}
						Update both halves in one go. We'll only ask if something needs a cable or a reset.
					{/if}
				</p>
			</div>

			<div class="space-y-5">
				{#if busy}
					<div
						class="progress-shell mx-auto w-52"
						role="progressbar"
						aria-valuenow={progress}
						aria-valuemin={0}
						aria-valuemax={100}
						aria-label={statusLine}
					>
						<div class="progress-face">
							<div class="progress-fill" style="width: {progress}%"></div>
						</div>
					</div>
				{/if}

				<Button
					size="lg"
					class="min-w-[13rem]"
					onclick={runFlash}
					disabled={busy || !isTauri()}
				>
					{busy ? statusLine.replace('…', '') : 'Flash keyboard'}
				</Button>

				{#if firmwareState.phase === 'error' && firmwareState.message}
					<p class="text-sm text-muted-foreground">{firmwareState.message}</p>
					<button
						type="button"
						class="font-mono text-[0.65rem] uppercase tracking-[0.14em] text-muted-foreground underline-offset-4 hover:text-foreground hover:underline"
						onclick={() => (firmwareState.showLog = !firmwareState.showLog)}
					>
						{firmwareState.showLog ? 'Hide details' : 'Show details'}
					</button>
				{/if}
			</div>

			{#if !busy}
				<button
					type="button"
					class="eyebrow transition-colors hover:text-foreground"
					onclick={() => (firmwareState.showAdvanced = !firmwareState.showAdvanced)}
				>
					{firmwareState.showAdvanced ? 'Hide advanced' : 'Advanced'}
				</button>
			{/if}
		</div>
	{/if}

	{#if firmwareState.showAdvanced && !inWizard && firmwareState.phase !== 'done'}
		<div class="surface-panel mx-auto mt-12 w-full max-w-lg text-left">
			<div class="surface-panel-inner space-y-4 p-5">
				{#if firmwareState.status?.leftUf2 || firmwareState.status?.rightUf2}
					<div class="space-y-1 font-mono text-[0.65rem] leading-relaxed text-muted-foreground">
						<div class="eyebrow">Flash from</div>
						{#if firmwareState.status?.leftUf2}
							<div class="truncate">L {firmwareState.status.leftUf2}</div>
						{/if}
						{#if firmwareState.status?.rightUf2}
							<div class="truncate">R {firmwareState.status.rightUf2}</div>
						{/if}
					</div>
				{/if}
				<label class="block space-y-1.5 text-sm">
					<span class="eyebrow">West build cache</span>
					<input
						class="field-key"
						bind:value={firmwareState.westRoot}
						onchange={() => refresh()}
						disabled={busy}
					/>
					<span class="text-xs text-muted-foreground">
						Only used when rebuilding. Flash copies
						<code class="text-foreground">artifacts/Lily58_*_oled.uf2</code>
						from this repo.
					</span>
				</label>
				<label class="flex items-center gap-2.5 text-sm text-muted-foreground">
					<input
						class="size-3.5 accent-foreground"
						type="checkbox"
						bind:checked={firmwareState.updateFirst}
						disabled={busy}
					/>
					Run west update before build
				</label>
				<label class="flex items-center gap-2.5 text-sm text-muted-foreground">
					<input
						class="size-3.5 accent-foreground"
						type="checkbox"
						bind:checked={firmwareState.forceBuild}
						disabled={busy}
					/>
					Force rebuild into artifacts/
				</label>
				<div class="flex flex-wrap gap-2 pt-1">
					<Button size="sm" variant="outline" onclick={refresh} disabled={busy}>Probe</Button>
					<Button size="sm" variant="outline" onclick={runUpdateOnly} disabled={busy}>
						West update
					</Button>
					<Button
						size="sm"
						variant="ghost"
						onclick={() => (firmwareState.showLog = !firmwareState.showLog)}
						disabled={!firmwareState.buildLog && !busy}
					>
						{firmwareState.showLog ? 'Hide log' : 'Build log'}
					</Button>
				</div>
			</div>
		</div>
	{/if}

	{#if firmwareState.showLog && firmwareState.buildLog}
		<pre
			bind:this={logEl}
			class="log-panel mx-auto mt-6 max-h-56 w-full max-w-lg overflow-auto p-4 text-left font-mono text-[0.65rem] leading-relaxed text-muted-foreground"
		>{firmwareState.buildLog}</pre>
	{/if}
</section>

<style>
	.flash-flow {
		animation: flash-enter 560ms cubic-bezier(0.22, 1, 0.36, 1) both;
	}

	.progress-shell {
		border-radius: 999px;
		background: var(--surface-shell);
		box-shadow: var(--shadow-shell);
		padding: 0.2rem;
	}

	.progress-face {
		height: 0.45rem;
		overflow: hidden;
		border-radius: 999px;
		background: linear-gradient(180deg, #141414 0%, #0f0f0f 100%);
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.5);
	}

	.progress-fill {
		height: 100%;
		border-radius: inherit;
		background: var(--surface-lit);
		box-shadow: 0 0 12px color-mix(in srgb, #f5f5f5 28%, transparent);
		transition: width 500ms cubic-bezier(0.22, 1, 0.36, 1);
	}

	.log-panel {
		border-radius: calc(var(--radius) + 2px);
		background: var(--surface-shell);
		box-shadow: var(--shadow-shell);
		border: 1px solid oklch(1 0 0 / 6%);
	}

	@keyframes flash-enter {
		from {
			opacity: 0;
			filter: blur(2px);
			transform: translateY(8px);
		}
		to {
			opacity: 1;
			filter: none;
			transform: none;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.flash-flow {
			animation: none;
		}

		.progress-fill {
			transition: none;
		}
	}
</style>
