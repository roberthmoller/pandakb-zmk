<script lang="ts">
	import * as Tabs from '#lib/components/ui/tabs/index.js';

	type Mode = 'learn' | 'train' | 'test' | 'firmware';

	type Props = {
		mode: Mode;
		onmode: (mode: Mode) => void;
	};

	let { mode, onmode }: Props = $props();
</script>

<Tabs.Root
	value={mode}
	onValueChange={(value) => {
		if (value === 'learn' || value === 'train' || value === 'test' || value === 'firmware') {
			onmode(value);
		}
	}}
>
	<Tabs.List aria-label="App mode">
		<Tabs.Trigger value="learn">Learn</Tabs.Trigger>
		<Tabs.Trigger value="train">Trainer</Tabs.Trigger>
		<Tabs.Trigger value="test">Test</Tabs.Trigger>
		<Tabs.Trigger value="firmware">Firmware</Tabs.Trigger>
	</Tabs.List>
</Tabs.Root>
