<script lang="ts" module>
	import { type VariantProps, tv } from "tailwind-variants";
	import { cn, type WithElementRef } from "#lib/utils.js";
	import type { HTMLAnchorAttributes, HTMLButtonAttributes } from "svelte/elements";

	export const buttonVariants = tv({
		base: "focus-visible:border-ring focus-visible:ring-ring/30 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 rounded-[calc(var(--radius)+2px)] border border-transparent bg-clip-padding font-mono text-[0.7rem] font-semibold uppercase tracking-[0.14em] focus-visible:ring-2 aria-invalid:ring-2 [&_svg:not([class*='size-'])]:size-3.5 group/button inline-flex shrink-0 items-center justify-center whitespace-nowrap outline-none select-none disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 transition-[transform,box-shadow,filter,background-color,color] duration-180 ease-[cubic-bezier(0.22,1,0.36,1)] active:not-aria-[haspopup]:translate-y-px active:not-aria-[haspopup]:scale-[0.98]",
		variants: {
			variant: {
				default:
					"bg-[image:var(--surface-lit)] text-primary-foreground shadow-[var(--shadow-lit)] hover:brightness-[1.04] hover:shadow-[0_1px_0_rgba(255,255,255,0.4),0_10px_22px_rgba(0,0,0,0.42),inset_0_1px_0_rgba(255,255,255,0.6)]",
				outline:
					"border-white/8 bg-[image:var(--surface-shell)] text-foreground shadow-[var(--shadow-shell)] hover:brightness-[1.06] hover:shadow-[var(--shadow-shell-hover)] hover:text-foreground",
				secondary:
					"bg-[image:var(--surface-face)] text-secondary-foreground shadow-[var(--shadow-shell)] hover:brightness-[1.06] hover:shadow-[var(--shadow-shell-hover)]",
				ghost:
					"bg-transparent text-muted-foreground shadow-none hover:bg-white/[0.04] hover:text-foreground hover:shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]",
				destructive:
					"bg-[image:linear-gradient(180deg,#3a3a3a_0%,#262626_100%)] text-destructive shadow-[var(--shadow-shell)] hover:brightness-110 focus-visible:ring-destructive/30",
				link: "rounded-none bg-transparent text-primary underline-offset-4 shadow-none hover:underline",
			},
			size: {
				default: "h-10 gap-1.5 px-6 has-data-[icon=inline-end]:pr-4 has-data-[icon=inline-start]:pl-4",
				xs: "h-7 gap-1 rounded-[calc(var(--radius))] px-3 text-[0.6rem] tracking-[0.12em] has-data-[icon=inline-end]:pr-2 has-data-[icon=inline-start]:pl-2 [&_svg:not([class*='size-'])]:size-3",
				sm: "h-9 gap-1 px-4 text-[0.65rem] has-data-[icon=inline-end]:pr-3 has-data-[icon=inline-start]:pl-3",
				lg: "h-12 gap-2 px-8 text-[0.72rem] tracking-[0.16em] has-data-[icon=inline-end]:pr-5 has-data-[icon=inline-start]:pl-5",
				icon: "size-10",
				"icon-xs": "size-7 rounded-[calc(var(--radius))] [&_svg:not([class*='size-'])]:size-3",
				"icon-sm": "size-9",
				"icon-lg": "size-11",
			},
		},
		defaultVariants: {
			variant: "default",
			size: "default",
		},
	});

	export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
	export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

	export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
		WithElementRef<HTMLAnchorAttributes> & {
			variant?: ButtonVariant;
			size?: ButtonSize;
		};
</script>

<script lang="ts">
	let {
		class: className,
		variant = "default",
		size = "default",
		ref = $bindable(null),
		href = undefined,
		type = "button",
		disabled,
		children,
		...restProps
	}: ButtonProps = $props();
</script>

{#if href}
	<a
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		href={disabled ? undefined : href}
		aria-disabled={disabled}
		role={disabled ? "link" : undefined}
		tabindex={disabled ? -1 : undefined}
		{...restProps}
	>
		{@render children?.()}
	</a>
{:else}
	<button
		bind:this={ref}
		data-slot="button"
		class={cn(buttonVariants({ variant, size }), className)}
		{type}
		{disabled}
		{...restProps}
	>
		{@render children?.()}
	</button>
{/if}
