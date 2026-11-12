<script lang="ts">
	import GameHeroHighway from '#lib/components/GameHeroHighway.svelte';
	import { Button } from '#lib/components/ui/button/index.js';
	import { bindingAt } from '#lib/keymap/resolve';
	import { boardState } from '#lib/state/board.svelte';
	import { typingState } from '#lib/state/typing.svelte';
	import { TRAINER_LAYOUTS, type TrainerLayoutId } from '#lib/typing/layouts';
	import { onMount } from 'svelte';

	onMount(() => {
		const onKeyDown = (event: KeyboardEvent) => {
			const target = event.target as HTMLElement | null;
			if (
				target &&
				(target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
			) {
				return;
			}
			typingState.handleKeydown(event);
		};
		window.addEventListener('keydown', onKeyDown);
		return () => window.removeEventListener('keydown', onKeyDown);
	});

	function labelForIndex(index: number): string {
		const overlay = typingState.overlayForIndex(index);
		if (overlay) return overlay.label;
		const binding = bindingAt(boardState.baseLayer, index);
		return binding?.label || `#${index}`;
	}

	function pressDisplay(value: string): string {
		const char = value === '␣' ? ' ' : value;
		if (!typingState.isGameLayout) return value === ' ' ? '␣' : value;
		if (!value) return '—';
		return typingState.displayForChar(char);
	}

	const last = $derived(typingState.lastPress);
	const judgment = $derived(last?.judgment);
</script>

<section class="space-y-3" aria-label="Typing trainer">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div class="space-y-2">
			<div class="font-mono text-[0.65rem] uppercase tracking-widest text-muted-foreground">
				{typingState.isGameLayout ? 'Game highway' : 'Type this'}
			</div>
			<div class="flex flex-wrap gap-1.5" role="group" aria-label="Trainer layout">
				{#each TRAINER_LAYOUTS as layout (layout.id)}
					<Button
						type="button"
						size="sm"
						variant={typingState.layoutId === layout.id ? 'default' : 'outline'}
						onclick={() => typingState.setLayout(layout.id as TrainerLayoutId)}
					>
						{layout.name}
					</Button>
				{/each}
			</div>
			{#if typingState.done}
				<p class="text-sm text-muted-foreground">
					{typingState.isGameLayout
						? 'Chart clear — Space or Enter for another.'
						: 'Line complete — Space or Enter for another.'}
				</p>
			{:else}
				<p class="max-w-xl text-sm text-muted-foreground">{typingState.layout.description}</p>
			{/if}
		</div>
		{#if typingState.done}
			<Button type="button" size="sm" onclick={() => typingState.reset()}>New line</Button>
		{/if}
	</div>

	{#if typingState.isGameLayout}
		<GameHeroHighway />
	{:else}
		<div
			class="rounded-lg bg-[#171717] px-5 py-6 font-mono text-2xl leading-relaxed tracking-wide md:text-3xl"
			aria-live="polite"
		>
			{#each [...typingState.prompt] as ch, i (i)}
				<span
					class="text-muted-foreground"
					class:text-foreground={i < typingState.cursor && typingState.typed[i] === ch}
					class:line-through={i < typingState.cursor && typingState.typed[i] !== ch}
					class:opacity-40={i < typingState.cursor && typingState.typed[i] !== ch}
					class:bg-white={i === typingState.cursor}
					class:text-black={i === typingState.cursor}
					class:rounded-sm={i === typingState.cursor}
					class:px-1={i === typingState.cursor}
					>{ch === ' ' ? '·' : ch}</span
				>
			{/each}
		</div>
	{/if}

	<div
		class="flex min-h-14 flex-wrap items-center gap-4 rounded-lg px-4 py-3 {last
			? last.correct
				? 'bg-[#1c1c1c]'
				: 'bg-[#2a2a2a]'
			: 'bg-transparent'}"
		aria-live="assertive"
	>
		{#if last}
			<span
				class="font-mono text-[0.65rem] uppercase tracking-widest {last.correct
					? 'text-foreground'
					: 'text-muted-foreground'}"
			>
				{#if judgment === 'perfect'}
					Perfect
				{:else if judgment === 'great'}
					Great
				{:else if judgment === 'overhit'}
					Overhit
				{:else if judgment === 'miss'}
					Miss
				{:else if last.correct}
					Hit
				{:else}
					Wrong
				{/if}
			</span>
			<span class="font-mono text-xl">
				you pressed <strong class="text-foreground">{pressDisplay(last.char)}</strong>
			</span>
			{#if !last.correct && last.expected}
				<span class="font-mono text-xl text-muted-foreground">
					expected <strong class="text-foreground">{pressDisplay(last.expected)}</strong>
				</span>
			{/if}
			{#if last.index !== null}
				<span class="font-mono text-sm text-muted-foreground">
					key {labelForIndex(last.index)}
				</span>
			{/if}
		{:else}
			<span class="font-mono text-sm text-muted-foreground">
				{#if typingState.isGameLayout}
					Notes scroll continuously — hit the matching key as gems cross the freeline.
				{:else}
					Start typing — next key is highlighted on the board.
				{/if}
			</span>
		{/if}
	</div>
</section>
