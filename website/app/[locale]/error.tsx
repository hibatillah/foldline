"use client";

import { useTranslations } from "next-intl";
import { useEffect } from "react";

import { LANDING } from "@/components/landing/landing-classes";
import { StatusPage } from "@/components/landing/status-page";
import { buttonVariants } from "@/components/ui/button";
import { Link } from "@/i18n/navigation";
import { cn } from "@/lib/utils";

const BUTTON = "h-11! gap-2! rounded-lg! px-5! text-[15px]! no-underline";

/** The fallback when a page throws; it carries no header or footer, so a broken one cannot break it too. */
export default function ErrorPage({ error, retry }: { error: unknown; retry: () => void }) {
	const t = useTranslations();

	useEffect(() => {
		console.error(error);
	}, [error]);

	return (
		<div
			className={`${LANDING} mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] min-h-[100vh] flex items-center font-sans text-foreground`}
		>
			<StatusPage code={t("error.code")} title={t("error.a")} text={t("error.b")}>
				<button type="button" onClick={() => retry()} className={cn(buttonVariants(), BUTTON)}>
					{t("error.retry")}
				</button>
				<Link
					href="/"
					className={cn(
						buttonVariants({ variant: "outline" }),
						BUTTON,
						"bg-white! text-foreground! hover:bg-muted!",
					)}
				>
					{t("error.home")}
				</Link>
			</StatusPage>
		</div>
	);
}
