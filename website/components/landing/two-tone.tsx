import type { ReactNode } from "react";

/** A heading whose first line is dark and second line muted. */
export function TwoTone({
	first,
	second,
	className = "text-[clamp(26px,2.4vw,32px)] leading-[1.22]",
}: {
	first: ReactNode;
	second: ReactNode;
	className?: string;
}) {
	return (
		<h2 className={`${className} m-0 font-medium tracking-[-0.025em] text-balance`}>
			{first}
			<br />
			<span className="text-muted-foreground">{second}</span>
		</h2>
	);
}
