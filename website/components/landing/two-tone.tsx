import type { ReactNode } from "react";

/** A heading whose first line is dark and second line muted. */
export function TwoTone({
	first,
	second,
	className = "h2",
}: {
	first: ReactNode;
	second: ReactNode;
	className?: string;
}) {
	return (
		<h2
			className={className}
			style={{ margin: 0, fontWeight: 500, letterSpacing: "-0.025em", textWrap: "balance" }}
		>
			{first}
			<br />
			<span style={{ color: "var(--muted-foreground)" }}>{second}</span>
		</h2>
	);
}
