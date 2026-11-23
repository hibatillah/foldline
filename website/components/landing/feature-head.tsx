import { useTranslations } from "next-intl";

import { TwoTone } from "./two-tone";

export type FeatureKey = "open" | "notice" | "plan" | "keep";

/** The numbered heading that opens each feature section. */
export function FeatureHead({ section, number }: { section: FeatureKey; number: string }) {
	const t = useTranslations();

	return (
		<div className="grid grid-cols-[repeat(2,minmax(0,1fr))] gap-x-12 gap-y-6 mb-[72px] max-[900px]:grid-cols-[minmax(0,1fr)] max-[640px]:mb-10 max-[640px]:gap-4">
			<p className="text-[clamp(24px,2.2vw,30px)] font-medium tracking-[-0.02em] m-0">
				<span className="text-muted-foreground">{number}</span>
				&nbsp;&nbsp;{t(`f.${section}.label`)}
			</p>
			<div>
				<TwoTone first={t(`f.${section}.a`)} second={t(`f.${section}.b`)} />
				<p className="max-w-[620px] text-[clamp(17px,1.5vw,21px)] leading-[1.55] text-muted-foreground mt-9 max-[640px]:mt-5">
					{t(`f.${section}.p`)}
				</p>
			</div>
		</div>
	);
}
