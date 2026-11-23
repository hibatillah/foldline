/** The base classes of every public page: the font, colors, selection color, link and focus styles, anchor offset. */
export const LANDING =
	"font-sans text-foreground bg-background antialiased selection:bg-[color-mix(in_oklch,var(--primary)_20%,transparent)] selection:text-foreground [&_a]:underline-offset-[3px] [&_a:not([data-slot]):hover]:text-primary [&_:focus-visible]:outline-2 [&_:focus-visible]:outline-offset-[3px] [&_:focus-visible]:outline-primary [&_section[id]]:scroll-mt-24";
