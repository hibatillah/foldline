import Image from "next/image";
import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";
import type en from "@/messages/en.json";
import { ExternalLink } from "./external-link";
import { FeedbackDialog } from "./feedback-dialog";
import { FooterWordmark } from "./footer-wordmark";
import { ExternalArrow } from "./external-arrow";

type Messages = typeof en;

/** A footer link: muted, with its arrow beside the label. */
const LINK = "inline-flex items-center gap-1.5 text-muted-foreground no-underline";

export function SiteFooter() {
	const t = useTranslations();
	const foot = t.raw("foot") as Messages["foot"];

	return (
		<footer className="relative overflow-hidden mt-[96px]">
			<FooterWordmark />
			<div className="mx-auto box-border max-w-[1280px] relative pt-[24px] pb-[clamp(110px,12.5vw,195px)] px-[clamp(16px,5vw,80px)] max-[640px]:pb-[28vw]">
				<div className="grid grid-cols-[minmax(0,2fr)_repeat(3,minmax(0,1fr))] gap-8 pb-7 max-[640px]:grid-cols-[repeat(2,minmax(0,1fr))] max-[640px]:gap-x-4 max-[640px]:gap-y-8 max-[640px]:[&>div:first-child]:col-span-2">
					<div className="flex flex-col gap-[10px]">
						<span className="flex items-center gap-[10px] font-semibold text-[16px]">
							<Image src="/logo.svg" alt="" width={22} height={22} className="rounded-[6px]" />
							Foldline
						</span>
						<span className="text-[14px] leading-[1.55] text-muted-foreground max-w-[260px]">
							{t("footTag")}
						</span>
					</div>
					{foot.map((fc) => (
						<div className="flex flex-col gap-[10px] text-[14px]" key={fc.h}>
							<span className="font-medium">{fc.h}</span>
							{fc.links.map((link) => {
								if (link.href === "dialog:feedback") {
									return <FeedbackDialog label={link.label} key={link.label} />;
								}
								// Anchors and pages stay on the site, so they work from the policy pages too.
								if (!link.href.startsWith("http")) {
									return (
										<Link
											href={link.href.startsWith("#") ? `/${link.href}` : link.href}
											className={LINK}
											key={link.label}
										>
											{link.label}
										</Link>
									);
								}
								return (
									<ExternalLink href={link.href} className={LINK} key={link.label}>
										{link.label}
										<ExternalArrow />
									</ExternalLink>
								);
							})}
						</div>
					))}
				</div>
				<div className="pt-[4px] text-[13px] text-muted-foreground">
					<span>{"© 2026 Hibatillah · GPL-3.0-or-later"}</span>
				</div>
			</div>
		</footer>
	);
}
