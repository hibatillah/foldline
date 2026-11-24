import type { ReactNode } from "react";

import { Reveal } from "./reveal";
import { PANEL, TEXTURE, TEXTURE_FILL } from "./textures";

/** The body of a status page (404, error): a notch showing the code, a headline and the actions. */
export function StatusPage({
	code,
	title,
	text,
	children,
}: {
	code: string;
	title: string;
	text: string;
	children: ReactNode;
}) {
	return (
		<Reveal className="mx-auto max-w-[720px] pb-6">
			<div
				className={`${PANEL} ${TEXTURE.henrikLight} ${TEXTURE_FILL} min-h-[200px] bg-[position:50%_40%]`}
			>
				<div className="box-border w-[min(360px,100%)] self-start rounded-[0_0_28px_28px] bg-black p-[20px_24px_24px] text-center font-mono text-[44px] font-medium tracking-[-0.02em] text-white">
					{code}
				</div>
			</div>
			<h1 className="mt-12 text-[clamp(30px,3vw,40px)] leading-[1.18] font-medium tracking-[-0.03em] text-balance">
				{title}
			</h1>
			<p className="mt-5 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground">
				{text}
			</p>
			<div className="mt-9 flex flex-wrap gap-3">{children}</div>
		</Reveal>
	);
}
