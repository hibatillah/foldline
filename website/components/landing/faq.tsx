import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type en from "@/messages/en.json";

type Messages = typeof en;

/** The link under an answer that points to the section covering it. */
const MORE_LINK =
	"mt-3 inline-flex items-center gap-1.5 text-[15px] font-medium text-primary no-underline hover:underline";

export function Faq() {
	const t = useTranslations();
	const faq = t.raw("faq") as Messages["faq"];

	return (
		<section
			id="faq"
			className="mx-auto box-border max-w-[1280px] pt-[clamp(64px,_7vw,_96px)] px-[clamp(16px,5vw,80px)]"
		>
			<div className="mb-[40px]">
				<h2 className="text-[clamp(26px,2.4vw,32px)] leading-[1.22] m-0 font-medium tracking-[-0.025em] text-balance">
					{t("faqTitle.a")}
					<br />
					<span className="text-muted-foreground">{t("faqTitle.b")}</span>
				</h2>
			</div>
			<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] items-start gap-x-12 gap-y-0 max-[640px]:grid-cols-[minmax(0,1fr)]">
				{faq.map((q) => (
					<details className="faq-item group border-t border-border py-5" key={q.q}>
						<summary className="flex cursor-pointer list-none justify-between gap-4 text-[17px] font-medium text-pretty [&::-webkit-details-marker]:hidden">
							{q.q}
							<svg
								className="mt-[3px] flex-none transition-transform duration-200 ease-[cubic-bezier(0.16,1,0.3,1)] group-open:rotate-45 motion-reduce:transition-none"
								aria-hidden="true"
								width="18"
								height="18"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								strokeWidth="2"
								strokeLinecap="round"
							>
								<path d="M12 5v14M5 12h14" />
							</svg>
						</summary>
						<p className="m-[12px_0_0] text-[16px] leading-[1.6] text-muted-foreground max-w-[60ch] text-pretty">
							{q.a}
						</p>
						{q.more &&
							(q.more.href.startsWith("/") ? (
								<Link href={q.more.href} className={MORE_LINK}>
									{q.more.label}
									<span aria-hidden="true">→</span>
								</Link>
							) : (
								<a href={q.more.href} className={MORE_LINK}>
									{q.more.label}
									<span aria-hidden="true">→</span>
								</a>
							))}
					</details>
				))}
			</div>
		</section>
	);
}
