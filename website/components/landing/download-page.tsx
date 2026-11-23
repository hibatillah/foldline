import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type en from "@/messages/en.json";

import { AutoDownload } from "./auto-download";
import { CtaLink } from "./cta-link";
import { PageShell } from "./page-shell";
import { WindowsIcon } from "./windows-icon";

type Messages = typeof en;

const REPOSITORY = "https://github.com/hibatillah/foldline";

/** The body text of the page: muted, with room between lines. */
const BODY = "m-0 mb-3 text-base leading-[1.65] text-muted-foreground";

/** The heading of a section below the steps. */
const HEADING = "mt-12 mb-2 text-xl font-semibold";

const EMPHASIS_PATTERN = /(\*\*[^*]+\*\*)/g;

/** Text where **words** stand out: the exact labels a visitor will see on the warning screens. */
function Emphasized({ text }: { text: string }) {
	return text.split(EMPHASIS_PATTERN).map((part) =>
		part.startsWith("**") ? (
			<strong
				key={part}
				className="rounded bg-[color-mix(in_oklch,var(--primary)_12%,transparent)] px-[5px] py-px font-semibold text-foreground"
			>
				{part.slice(2, -2)}
			</strong>
		) : (
			part
		),
	);
}

/** The page every Download button opens: it starts the download and explains the warnings that follow. */
export function DownloadPage() {
	const t = useTranslations();
	const content = t.raw("download") as Messages["download"];

	return (
		<PageShell>
			<AutoDownload />
			<article className="max-w-[720px]">
				<h1 className="text-[clamp(30px,3vw,40px)] leading-[1.18] m-[0_0_8px] font-semibold">
					{content.title}
				</h1>
				<p className="m-[0_0_24px] text-[17px] leading-[1.6]">{content.subtitle}</p>
				<div className="flex flex-wrap items-center gap-[16px]">
					<CtaLink href="/api/download" size="lg" variant="primary">
						<WindowsIcon />
						{content.again}
					</CtaLink>
					<span className="text-[14px] text-muted-foreground">{content.meta}</span>
				</div>

				<h2 className={HEADING}>{content.stepsTitle}</h2>
				<p className={BODY}>{content.stepsIntro}</p>
				<ol className="[list-style:none] m-[16px_0_0] p-0 grid gap-[20px]">
					{content.steps.map((step, index) => (
						<li key={step.h} className="flex gap-[16px]">
							<span
								aria-hidden="true"
								className="flex-none w-[28px] h-[28px] rounded-full grid place-items-center text-[14px] font-semibold bg-primary text-primary-foreground"
							>
								{index + 1}
							</span>
							<div>
								<h3 className="m-[2px_0_6px] text-[16px] font-semibold">{step.h}</h3>
								<p className={`${BODY} mb-0`}>
									<Emphasized text={step.p} />
								</p>
							</div>
						</li>
					))}
				</ol>

				<h2 className={HEADING}>{content.whyTitle}</h2>
				<p className={BODY}>{content.why}</p>

				<h2 className={HEADING}>{content.safeTitle}</h2>
				{content.safe.map((paragraph) => (
					<p key={paragraph} className={BODY}>
						{paragraph}
					</p>
				))}
				<div className="flex flex-wrap gap-[12px] mt-[20px]">
					<CtaLink href={REPOSITORY} size="lg" variant="outline">
						{content.source}
					</CtaLink>
					<Link
						href="/privacy"
						className="inline-flex items-center h-[44px] p-[0_4px] text-[15px] text-primary"
					>
						{content.privacy}
					</Link>
				</div>
			</article>
		</PageShell>
	);
}
