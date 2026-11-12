<script lang="ts">
	import { Button } from '#lib/components/ui/button/index.js';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { boardState } from '#lib/state/board.svelte';
	import { parseKeymap } from '#lib/keymap/parseKeymap';

	let githubUrl = $state('https://github.com/roberthmoller/pandakb-zmk/tree/zmk-for-lily');
	let status = $state<string | null>(null);
	let busy = $state(false);
	let fileInput: HTMLInputElement | undefined = $state();

	const origin = $derived(boardState.configOrigin);
	const layerCount = $derived(boardState.board.layers.length);
	const keyCount = $derived(boardState.board.keys.length);

	function applyParsed(
		keymapText: string,
		origin: Parameters<typeof boardState.applyLayers>[1]
	) {
		const parsed = parseKeymap(keymapText, { keyCount });
		boardState.applyLayers(parsed.layers, origin, parsed.warnings);
		const warn =
			parsed.warnings.length > 0 ? ` (${parsed.warnings.length} warning${parsed.warnings.length === 1 ? '' : 's'})` : '';
		status = `Loaded ${parsed.layers.length} layers${warn}.`;
	}

	async function loadFromGitHub() {
		busy = true;
		status = null;
		try {
			const res = await fetch(`/api/config/github?url=${encodeURIComponent(githubUrl.trim())}`);
			const data = (await res.json().catch(() => ({}))) as {
				message?: string;
				keymap?: string;
				ref?: { owner: string; repo: string; branch: string };
				source?: string;
			};
			if (!res.ok || !data.keymap || !data.ref || !data.source) {
				status = data.message ?? `GitHub fetch failed (${res.status})`;
				return;
			}
			applyParsed(data.keymap, {
				kind: 'github',
				label: `${data.ref.owner}/${data.ref.repo}@${data.ref.branch}`,
				url: data.source
			});
		} catch (e) {
			status = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	async function onFileChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		busy = true;
		status = null;
		try {
			const text = await file.text();
			applyParsed(text, { kind: 'upload', label: file.name, fileName: file.name });
		} catch (e) {
			status = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
			input.value = '';
		}
	}

	function resetBuiltin() {
		boardState.resetToBuiltin();
		status = 'Restored PandaKB stock keymap.';
	}
</script>

<section
	class="flex flex-col gap-3 border border-border bg-muted/20 p-4"
	aria-label="Keymap source"
>
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div class="flex flex-wrap items-center gap-2">
			<span class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground"
				>Keymap</span
			>
			<Badge variant="secondary">
				{#if origin.kind === 'builtin'}
					{origin.label}
				{:else if origin.kind === 'github'}
					{origin.label}
				{:else}
					{origin.fileName}
				{/if}
			</Badge>
			<span class="font-mono text-xs text-muted-foreground">{layerCount} layers</span>
		</div>
		{#if origin.kind !== 'builtin'}
			<Button type="button" size="xs" variant="ghost" onclick={resetBuiltin} disabled={busy}>
				Stock
			</Button>
		{/if}
	</div>

	<div class="flex flex-col gap-2 sm:flex-row sm:items-center">
		<label class="sr-only" for="github-keymap-url">GitHub config repo</label>
		<input
			id="github-keymap-url"
			type="url"
			bind:value={githubUrl}
			placeholder="https://github.com/owner/repo/tree/branch"
			class="h-9 min-w-0 flex-1 border border-border bg-background px-3 font-mono text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/30"
			disabled={busy}
			onkeydown={(e) => {
				if (e.key === 'Enter') {
					e.preventDefault();
					void loadFromGitHub();
				}
			}}
		/>
		<Button type="button" size="sm" variant="outline" onclick={() => void loadFromGitHub()} disabled={busy}>
			{busy ? 'Loading…' : 'Load repo'}
		</Button>
		<input
			bind:this={fileInput}
			type="file"
			accept=".keymap,text/plain"
			class="sr-only"
			onchange={onFileChange}
		/>
		<Button
			type="button"
			size="sm"
			variant="secondary"
			disabled={busy}
			onclick={() => fileInput?.click()}
		>
			Upload .keymap
		</Button>
	</div>

	{#if status}
		<p class="font-mono text-xs text-muted-foreground">{status}</p>
	{/if}
	{#if boardState.configWarnings.length > 0}
		<ul class="list-inside list-disc font-mono text-xs text-amber-600/90 dark:text-amber-400/90">
			{#each boardState.configWarnings as warning (warning)}
				<li>{warning}</li>
			{/each}
		</ul>
	{/if}
</section>
