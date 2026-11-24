import { useTranslations } from "next-intl";

import { AnatomyDiagram } from "./anatomy-diagram";
import { Reveal } from "./reveal";
import { TwoTone } from "./two-tone";

/** The notch from the front: the frame the host draws and the one area a page owns. */
export function Anatomy() {
	const t = useTranslations();
	const part = (key: "edge" | "host" | "pills" | "clock" | "status" | "page") => ({
		t: t(`anatomy.${key}.t`),
		d: t(`anatomy.${key}.d`),
	});

	return (
		<section
			id="anatomy"
			className="mx-auto box-border max-w-[1280px] px-[clamp(16px,5vw,80px)] pt-[clamp(120px,13vw,200px)] max-[640px]:pt-24"
		>
			<Reveal className="mb-[72px] grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-4 max-[640px]:gap-4">
				<p className="m-0 text-[clamp(24px,2.2vw,30px)] font-medium tracking-[-0.02em]">
					<span className="text-muted-foreground">01</span>
					{"  "}
					{t("anatomy.label")}
				</p>
				<div>
					<TwoTone first={t("anatomy.a")} second={t("anatomy.b")} />
					<p className="mt-9 max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground max-[640px]:mt-5">
						{t("anatomy.p")}
					</p>
				</div>
			</Reveal>
			<AnatomyDiagram
				legend={t("anatomy.legend")}
				labels={{
					edge: part("edge"),
					host: part("host"),
					pills: part("pills"),
					clock: part("clock"),
					status: part("status"),
					page: part("page"),
				}}
			/>
		</section>
	);
}
