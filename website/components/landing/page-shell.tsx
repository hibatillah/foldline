import type { ReactNode } from "react";

import { LANDING } from "./landing-classes";
import { Reveal } from "./reveal";
import { SiteFooter } from "./site-footer";
import { SiteNav } from "./site-nav";

/** The frame of a page that is not the landing page: the site header, the content and the site footer. */
export function PageShell({ children }: { children: ReactNode }) {
	return (
		<div className={LANDING}>
			<SiteNav />
			<main className="mx-auto box-border max-w-[1280px] pt-[56px] px-[clamp(16px,5vw,80px)]">
				<Reveal distance={14}>{children}</Reveal>
			</main>
			<SiteFooter />
		</div>
	);
}
