import { useTranslations } from "next-intl";

import type en from "@/messages/en.json";

import { PageShell } from "./page-shell";
import { Reveal } from "./reveal";

type LegalDocument = Messages["privacy"];
type Messages = typeof en;

const LINK_PATTERN = /(\[[^\]]+\]\([^)]+\))/g;
const LINK_PARTS = /^\[([^\]]+)\]\(([^)]+)\)$/;

/** Text where [label](url) becomes an outside link. */
function RichText({ text }: { text: string }) {
	return text.split(LINK_PATTERN).map((part) => {
		const link = LINK_PARTS.exec(part);
		if (!link) return part;
		return (
			<a key={part} href={link[2]} target="_blank" rel="noreferrer" className="text-primary">
				{link[1]}
			</a>
		);
	});
}

/** A policy page: a slim header, the title and dated intro, numbered sections and the site footer. */
export function LegalPage({ document }: { document: "privacy" | "terms" }) {
	const t = useTranslations();
	const content = t.raw(document) as LegalDocument;

	return (
		<PageShell>
			<article className="max-w-[720px] mx-auto">
				<h1 className="text-[clamp(30px,3vw,40px)] leading-[1.18] m-[0_0_8px] font-semibold">
					{content.title}
				</h1>
				<p className="m-[0_0_24px] text-[14px] text-muted-foreground">
					{t("legalUpdated")}: {t("legalUpdatedOn")}
				</p>
				<p className="m-[0_0_40px] text-[17px] leading-[1.6]">{content.intro}</p>
				{content.sections.map((section, index) => (
					<Reveal key={section.h} className="mb-[36px]">
						<h2 className="leading-[1.25] m-[0_0_12px] font-semibold text-[20px]">
							{index + 1}. {section.h}
						</h2>
						{section.p.map((paragraph) => (
							<p
								key={paragraph}
								className="m-[0_0_12px] text-[16px] leading-[1.65] text-muted-foreground"
							>
								<RichText text={paragraph} />
							</p>
						))}
						{"ul" in section && section.ul && (
							<ul className="m-[0_0_12px] pl-[22px] [list-style:disc] text-[16px] leading-[1.65] text-muted-foreground">
								{section.ul.map((item) => (
									<li key={item} className="mb-[6px]">
										<RichText text={item} />
									</li>
								))}
							</ul>
						)}
					</Reveal>
				))}
			</article>
		</PageShell>
	);
}
