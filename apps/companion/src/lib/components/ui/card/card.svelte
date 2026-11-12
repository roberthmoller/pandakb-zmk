<script lang="ts">
	import { cn, type WithElementRef } from "#lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		size = "default",
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { size?: "default" | "sm" } = $props();
</script>

<div
	bind:this={ref}
	data-slot="card"
	data-size={size}
	class={cn(
		"group/card text-card-foreground flex flex-col gap-(--card-spacing) overflow-hidden rounded-[calc(var(--radius)+2px)] bg-[image:var(--surface-shell)] py-(--card-spacing) text-sm shadow-[var(--shadow-shell)] [--card-spacing:--spacing(6)] data-[size=sm]:[--card-spacing:--spacing(4)] *:[img:first-child]:rounded-none *:[img:last-child]:rounded-none",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</div>
