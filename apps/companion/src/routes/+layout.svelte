<script lang="ts">
	import './layout.css';
	import favicon from '#lib/assets/favicon.svg';
	import { loadFirmwareKeymap } from '#lib/firmware/loadKeymap';
	import { isTauri } from '#lib/firmware/tauri';
	import type { LayoutProps } from './$types';
	import { onMount } from 'svelte';

	let { children }: LayoutProps = $props();

	onMount(() => {
		if (!isTauri()) return;
		void loadFirmwareKeymap();
	});
</script>

<svelte:head>
	<title>PandaKB — Build & flash</title>
	<meta
		name="description"
		content="Build and flash PandaKB Lily58 companion OLED firmware."
	/>
	<link rel="preconnect" href="https://fonts.googleapis.com" />
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=DM+Sans:ital,opsz,wght@0,9..40,400;0,9..40,500;0,9..40,600;0,9..40,700;1,9..40,400&family=IBM+Plex+Mono:wght@400;500;600&display=swap"
		rel="stylesheet"
	/>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
