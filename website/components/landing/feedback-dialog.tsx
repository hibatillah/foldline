"use client";

import { Dialog } from "@base-ui/react/dialog";
import posthog from "posthog-js";
import { useTranslations } from "next-intl";
import { useState, type FormEvent } from "react";

import { ExternalLink } from "./external-link";

const SURVEY_ID = process.env.NEXT_PUBLIC_POSTHOG_SURVEY_ID;
const ISSUES_URL = "https://github.com/hibatillah/foldline/issues";

type Status = "idle" | "sending" | "sent" | "failed";
type SurveyDefinition = { id: string; questionIds: string[] };

/** The look shared by the fields and the close button. */
const FIELD =
	"box-border w-full rounded-lg border border-[color-mix(in_oklch,var(--foreground)_14%,transparent)] bg-background px-3 py-2.5 text-sm text-foreground [font:inherit]";

/** The text of one form field, or an empty string for a missing one. */
function fieldText(form: FormData, name: string) {
	const value = form.get(name);
	return typeof value === "string" ? value : "";
}

/** Looks the survey up by its id, so the question ids never live in the code. */
function loadSurvey(): Promise<SurveyDefinition | null> {
	return new Promise((resolve) => {
		if (!SURVEY_ID || !posthog.__loaded) return resolve(null);
		posthog.getSurveys((surveys) => {
			const found = surveys.find((survey) => survey.id === SURVEY_ID);
			resolve(
				found
					? { id: found.id, questionIds: found.questions.flatMap((q) => (q.id ? [q.id] : [])) }
					: null,
			);
		}, true);
	});
}

/** A footer button that opens a bug report form; the answer goes to a PostHog survey. */
export function FeedbackDialog({ label }: { label: string }) {
	const t = useTranslations();
	const [status, setStatus] = useState<Status>("idle");

	async function send(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = new FormData(event.currentTarget);
		setStatus("sending");
		const survey = await loadSurvey();
		if (!survey || survey.questionIds.length === 0) return setStatus("failed");

		// The survey has two open-text questions: what went wrong, then an optional way to reply.
		const [messageId, contactId] = survey.questionIds;
		posthog.capture("survey sent", {
			$survey_id: survey.id,
			$survey_submission_id: crypto.randomUUID(),
			$survey_completed: true,
			[`$survey_response_${messageId}`]: fieldText(form, "message"),
			...(contactId ? { [`$survey_response_${contactId}`]: fieldText(form, "contact") } : {}),
		});
		setStatus("sent");
	}

	return (
		<Dialog.Root onOpenChange={() => setStatus("idle")}>
			<Dialog.Trigger className="cursor-pointer border-0 bg-transparent p-0 text-left text-muted-foreground [font:inherit]">
				{label}
			</Dialog.Trigger>
			<Dialog.Portal>
				<Dialog.Backdrop className="fixed inset-0 z-[50] bg-[rgba(15,23,42,0.4)]" />
				<Dialog.Popup className="fixed z-[51] top-[50%] left-[50%] [transform:translate(-50%,_-50%)] w-[min(440px,_calc(100vw_-_32px))] box-border p-[24px] rounded-[16px] bg-card text-foreground font-sans [box-shadow:0_24px_64px_-16px_rgba(15,23,42,0.45)]">
					<Dialog.Title className="m-0 text-[18px] font-semibold">
						{t("feedback.title")}
					</Dialog.Title>
					<Dialog.Description className="m-[8px_0_16px] text-[14px] text-muted-foreground">
						{t("feedback.text")}
					</Dialog.Description>
					{status === "sent" ? (
						<>
							<p className="text-[14px] m-[0_0_16px]">{t("feedback.thanks")}</p>
							<Dialog.Close className={`${FIELD} w-auto! cursor-pointer`}>
								{t("feedback.close")}
							</Dialog.Close>
						</>
					) : (
						<form onSubmit={send} className="grid gap-[12px]">
							<label className="grid gap-[6px] text-[13px] font-medium">
								{t("feedback.message")}
								<textarea
									name="message"
									placeholder={t("feedback.messagePlaceholder")}
									required
									maxLength={2000}
									rows={5}
									className={`${FIELD} resize-y`}
								/>
							</label>
							<label className="grid gap-[6px] text-[13px] font-medium">
								{t("feedback.contact")}
								<input
									name="contact"
									placeholder={t("feedback.contactPlaceholder")}
									maxLength={200}
									autoComplete="email"
									className={FIELD}
								/>
							</label>
							{status === "failed" && (
								<p role="alert" className="m-0 text-[13px] text-destructive">
									{t("feedback.failed")}
								</p>
							)}
							<div className="flex flex-wrap items-center justify-between gap-[12px]">
								<ExternalLink href={ISSUES_URL} className="text-[13px] text-primary">
									{t("feedback.issue")}
								</ExternalLink>
								<button
									type="submit"
									disabled={status === "sending"}
									className="h-[36px] p-[0_16px] [border:0] rounded-[8px] bg-primary text-primary-foreground text-[14px] font-medium cursor-pointer"
								>
									{t("feedback.send")}
								</button>
							</div>
						</form>
					)}
				</Dialog.Popup>
			</Dialog.Portal>
		</Dialog.Root>
	);
}
