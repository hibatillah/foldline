import type { ReactNode } from "react";

import { Flares } from "./open-notch";
import { NOTCH_WIDTH_CLASS } from "./sizes";

/** A notification card that drops out of the pill. */
export function Banner({
	notch,
	children,
}: {
	notch: "notification" | "download";
	children: ReactNode;
}) {
	return (
		<div
			className={`relative box-border rounded-[0_0_18px_18px] bg-black p-[12px_20px_16px] text-left font-sans text-white ${NOTCH_WIDTH_CLASS[notch]}`}
		>
			<Flares size={6} />
			{children}
		</div>
	);
}
