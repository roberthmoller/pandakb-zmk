<script lang="ts">
	import { buttonVariants } from '#lib/components/ui/button/index.js';
	import { boardState } from '#lib/state/board.svelte';
	import { cn } from '#lib/utils.js';
</script>

<footer
	class="mt-auto flex flex-wrap gap-x-4 gap-y-2 border-t border-white/10 py-4"
	aria-label="Firmware tools"
>
	{#each boardState.board.sources as source (source.url)}
		<a
			href={source.url}
			target="_blank"
			rel="noreferrer"
			class={cn(buttonVariants({ variant: 'link', size: 'sm' }), 'px-0')}
		>
			{source.title}
		</a>
	{/each}
</footer>
