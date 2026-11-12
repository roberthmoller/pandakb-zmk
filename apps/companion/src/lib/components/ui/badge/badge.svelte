<script lang="ts" module>
	import { type VariantProps, tv } from "tailwind-variants";

	export const badgeVariants = tv({
		base: "gap-1.5 rounded-[calc(var(--radius)-2px)] border px-2 py-0.5 font-mono text-[0.6rem] font-semibold uppercase tracking-[0.14em] transition-[color,background-color,box-shadow,border-color] duration-160 has-data-[icon=inline-end]:pr-1.5 has-data-[icon=inline-start]:pl-1.5 [&>svg]:size-3! group/badge inline-flex w-fit shrink-0 items-center justify-center overflow-hidden whitespace-nowrap focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 [&>svg]:pointer-events-none",
		variants: {
			variant: {
				default:
					"border-white/10 bg-[image:var(--surface-face)] text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] [a]:hover:brightness-110",
				secondary:
					"border-white/8 bg-white/[0.03] text-muted-foreground [a]:hover:text-foreground",
				destructive:
					"border-white/12 bg-white/[0.06] text-destructive [a]:hover:text-destructive/80 focus-visible:ring-destructive/20",
				outline:
					"border-white/12 bg-transparent text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.04)] [a]:hover:bg-white/[0.04]",
				ghost: "border-transparent bg-transparent px-0 text-muted-foreground hover:text-foreground",
				link: "border-transparent bg-transparent px-0 text-foreground underline-offset-4 hover:underline",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	});

	export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
	import { cn, type WithElementRef } from "#lib/utils.js";
	import type { HTMLAnchorAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		href,
		class: className,
		variant = "default",
		children,
		...restProps
	}: WithElementRef<HTMLAnchorAttributes> & {
		variant?: BadgeVariant;
	} = $props();
</script>

<svelte:element
	this={href ? "a" : "span"}
	bind:this={ref}
	data-slot="badge"
	{href}
	class={cn(badgeVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</svelte:element>
