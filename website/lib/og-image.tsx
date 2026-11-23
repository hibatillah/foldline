import { readFile } from "node:fs/promises";
import { join } from "node:path";

import { ImageResponse } from "next/og";
import { getTranslations } from "next-intl/server";

import { windowsLogoPath } from "@/components/landing/windows-icon";
import type { routing } from "@/i18n/routing";

export const ogSize = { width: 1200, height: 630 };

const logoSource = `data:image/svg+xml;base64,${await readFile(
	join(process.cwd(), "public/logo.svg"),
	"base64",
)}`;

/** The Windows logo of the download button, as in the hero. */
function WindowsMark() {
	return (
		<svg width="28" height="28" viewBox="0 0 24 24" fill="#ffffff">
			<path d={windowsLogoPath} />
		</svg>
	);
}

/** The shared image of a page: the hero headline, the download button and the open media notch. */
export async function renderOgImage(locale: (typeof routing.locales)[number]) {
	const t = await getTranslations({ locale });

	return new ImageResponse(
		<div tw="relative flex h-full w-full flex-col bg-white px-16 pt-12">
			<div tw="flex items-center">
				{/* eslint-disable-next-line @next/next/no-img-element */}
				<img src={logoSource} width={48} height={48} alt="" tw="rounded-xl" />
				<div tw="ml-4 text-[30px] font-semibold text-slate-900">Foldline</div>
			</div>
			<div tw="mt-8 flex max-w-[1000px] text-[38px] leading-[1.15] text-slate-500">
				{t("hero.b")}
			</div>
			<div tw="mt-2 flex text-[52px] font-semibold leading-[1.1] text-slate-900">{t("hero.a")}</div>
			<div tw="mt-8 flex items-center">
				<div tw="flex items-center rounded-2xl bg-blue-600 px-7 py-4 text-[26px] text-white">
					<WindowsMark />
					<div tw="ml-3 flex">{t("hero.cta")}</div>
				</div>
				<div tw="ml-6 flex text-[22px] text-slate-500">Windows 10 · 11</div>
			</div>
			<div
				tw="absolute bottom-0 left-16 right-16 flex h-[200px] justify-center rounded-t-[28px]"
				style={{ backgroundImage: "linear-gradient(135deg, #eaf2ff 0%, #d4e4ff 100%)" }}
			>
				<div tw="flex h-[150px] w-[560px] items-center rounded-b-[44px] bg-black px-8">
					<div
						tw="flex h-[96px] w-[96px] rounded-2xl"
						style={{ backgroundImage: "linear-gradient(135deg, #4aa3ff 0%, #1c5fe0 100%)" }}
					/>
					<div tw="ml-6 flex flex-1 flex-col">
						<div tw="flex h-[14px] w-[200px] rounded-full bg-white" />
						<div tw="mt-3 flex h-[10px] w-[140px] rounded-full bg-neutral-500" />
						<div tw="mt-6 flex h-[8px] w-full rounded-full bg-neutral-700">
							<div tw="flex h-[8px] w-[45%] rounded-full bg-white" />
						</div>
					</div>
				</div>
			</div>
		</div>,
		ogSize,
	);
}
