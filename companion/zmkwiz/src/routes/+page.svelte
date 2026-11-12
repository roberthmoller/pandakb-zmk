<script lang="ts">
	import ConfigSource from '#lib/components/ConfigSource.svelte';
	import ExternalTools from '#lib/components/ExternalTools.svelte';
	import FirmwareHub from '#lib/components/firmware/FirmwareHub.svelte';
	import KeyboardViz from '#lib/components/KeyboardViz.svelte';
	import LearnStage from '#lib/components/LearnStage.svelte';
	import LearnStats from '#lib/components/LearnStats.svelte';
	import ModeSwitcher from '#lib/components/ModeSwitcher.svelte';
	import TestStage from '#lib/components/TestStage.svelte';
	import TestStats from '#lib/components/TestStats.svelte';
	import TrainerStats from '#lib/components/TrainerStats.svelte';
	import TypingTrainer from '#lib/components/TypingTrainer.svelte';
	import { Badge } from '#lib/components/ui/badge/index.js';
	import { boardState } from '#lib/state/board.svelte';
	import { testState } from '#lib/state/test.svelte';
	import { typingState } from '#lib/state/typing.svelte';
	import { onMount } from 'svelte';

	type Mode = 'learn' | 'train' | 'test' | 'firmware';

	let mode = $state<Mode>('learn');

	function setMode(next: Mode) {
		mode = next;
		if (next === 'train') {
			boardState.setLayer('base');
			boardState.pressedIndices = [];
			typingState.reset(typingState.prompt);
		} else if (next === 'test') {
			boardState.setLayer('base');
			boardState.pressedIndices = [];
			testState.reset();
			testState.markRightRaise();
		} else if (next !== 'firmware') {
			boardState.setLayer('base');
		}
	}

	onMount(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			if (mode === 'learn') boardState.handlePhysicalKeyDown(event);
			else if (mode === 'test') testState.handleKeyDown(event);
		};
		const onKeyUp = (event: KeyboardEvent) => {
			if (mode === 'learn') boardState.handlePhysicalKeyUp(event);
			else if (mode === 'test') testState.handleKeyUp(event);
		};
		window.addEventListener('keydown', onKeyDown);
		window.addEventListener('keyup', onKeyUp);
		return () => {
			window.removeEventListener('keydown', onKeyDown);
			window.removeEventListener('keyup', onKeyUp);
		};
	});
</script>

<div class="flex min-h-svh flex-col">
	<main class="mx-auto flex w-[min(1600px,calc(100%-1.5rem))] flex-1 flex-col py-6">
		<header class="mb-8 flex flex-col gap-3">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex items-center gap-2">
					<Badge variant="secondary">{boardState.board.vendor}</Badge>
					<span class="font-mono text-xs uppercase tracking-widest text-muted-foreground"
						>ZMK wireless</span
					>
				</div>
				<ModeSwitcher {mode} onmode={setMode} />
			</div>
			<h1 class="font-heading text-4xl font-semibold tracking-tight md:text-5xl">ZmkWiz</h1>
			<p class="max-w-xl text-muted-foreground">
				{#if mode === 'learn'}
					Press keys to highlight them on the Lily58 and inspect every layer. Lower / Raise switch
					automatically from layer keys; on a laptop hold Left Alt / Right Alt to preview them.
				{:else if mode === 'train'}
					Type sentences, or switch to WoW / LoL for a scrolling Guitar Hero highway with hit windows.
				{:else if mode === 'test'}
					Find a dead switch: mark the suspect, mash the board, then follow the multimeter steps.
				{:else}
					Live Studio RPC remap in-app, or rebuild and flash from your local west tree.
				{/if}
			</p>
		</header>

		<div class="flex min-w-0 flex-1 flex-col gap-8">
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				{boardState.board.vendor}
				{boardState.board.name} · {mode === 'train'
					? 'trainer'
					: mode === 'test'
						? 'hardware test'
						: mode === 'firmware'
							? 'firmware'
							: 'learn'}
			</div>

			{#if mode === 'learn' || mode === 'train'}
				<ConfigSource />
			{/if}

			{#if mode === 'train'}
				<TypingTrainer />
				<div class="py-2">
					<KeyboardViz training />
				</div>
				<TrainerStats />
			{:else if mode === 'test'}
				<TestStage />
				<div class="py-2">
					<KeyboardViz testing />
				</div>
				<TestStats />
			{:else if mode === 'firmware'}
				<FirmwareHub />
			{:else}
				<LearnStage />
				<div class="py-2">
					<KeyboardViz />
				</div>
				<LearnStats />
			{/if}
		</div>

		<ExternalTools />
	</main>
</div>
