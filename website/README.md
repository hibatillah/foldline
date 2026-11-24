# Foldline website

The landing page of Foldline. Next.js (App Router), Tailwind CSS 4 and shadcn/ui (preset `b1ZzrZeZG`, Base UI). It is a separate program from the Rust workspace: Cargo ignores this folder.

## Run

```bash
bun install
bun run dev        # http://localhost:3000
bun run build
bun check && bun run lint && bun run format:check && bun test
```

Add a component with `bunx shadcn@latest add <name>`.

## Analytics (PostHog)

`instrumentation-client.ts` starts PostHog when `NEXT_PUBLIC_POSTHOG_KEY` is set; without it nothing is sent. Copy `.env.example` to `.env.local` and fill it (set `NEXT_PUBLIC_POSTHOG_HOST` to the EU host for an EU project). On Vercel, add the same two variables to the project.

- Requests go through `/ingest` (a rewrite in `next.config.ts`), so blockers that match posthog.com do not drop them.
- On: pageviews and time on page, autocapture of clicks (buttons and links), heatmaps, exceptions, web vitals.
- Off: session replay; anonymous visitors get no person profile (`person_profiles: "identified_only"`).
- Consent: `cookieless_mode: "always"` stores nothing on the device and counts visits with a hash PostHog makes on its servers, so the site has no consent banner. It cannot recognise returning visitors across days. This needs **Settings > Project > Web analytics > Enable cookieless tracking** turned on in PostHog. Using cookies again means a banner and a new privacy page.
- Bug reports: the footer's Report a bug dialog (`components/landing/feedback-dialog.tsx`) sends a `survey sent` event to an API survey with two open-text questions (what happened, optional contact). Create that survey in PostHog and put its id in `NEXT_PUBLIC_POSTHOG_SURVEY_ID`.
- The privacy page (`messages/*.json`, key `privacy`) states the PostHog US host and a 24-month retention; change the PostHog setting and the page together.
- Language: `/` is always English and `/id` Indonesian; the browser language is ignored and no locale cookie is set (`i18n/routing.ts`).
- A named event: `trackEvent("download_clicked", { channel: "github" })` from `lib/analytics.ts`; add the event name to `SiteEvent` first.

## Design

The accent is Foldline blue (`#2563eb` light, `#4c9aff` dark, from `docs/reference/design-rules.md`) over the shadcn preset's neutrals, in Geist. The page is light only (`forcedTheme="light"`); the design has no dark version.

The approved design (desktop 1440 px and mobile 390 px boards, the scripts that made them and the copy) is in `.superpowers/design/landing-v24/`. The notch drawings are components in `components/notch/`, the light elements and sections in `components/landing/`, and every style is a Tailwind class in the component that uses it. `app/globals.css` is the only CSS file: it holds the Tailwind imports and the shadcn color tokens. A value computed at run time (none today) would be the one reason for an inline `style`.

## Languages

next-intl serves English at `/` and Indonesian at `/id`, both built statically (`i18n/routing.ts`, `proxy.ts`). The copy lives in `messages/en.json` and `messages/id.json`. To add a key, add it to both files, then run `bun test`: it fails when the two files differ or when a message holds an unmeasured number such as "50 MB".

## SEO

- `lib/site.ts` holds the public address (`https://fold.hibatillah.com`) and the author; change the address there if the domain moves.
- `lib/seo.ts` (`pageMetadata`) gives each page its title, description, self-canonical, `hreflang` alternates (with `x-default`), Open Graph and Twitter tags. The download page is `noindex` and left out of the sitemap.
- `app/opengraph-image.jpg` and `twitter-image.jpg` are the one share image of every page and language: the first three rows of the landing page's feature grid, 1200 × 630 at 2×, with their `.alt.txt` texts. Capture it again after the grid changes.
- `lib/structured-data.ts` writes the home page JSON-LD: website, software application, author and the FAQ.
- `app/robots.ts` and `app/sitemap.ts` are generated; `public/llms.txt` is a hand-written summary for AI assistants. Keep its facts in step with the home page.
- Icons: `app/favicon.ico`, `app/icon.png` and `app/apple-icon.png` come from `assets/branding/tile-stable.*`.

## Deploy

Vercel, with the project's root directory set to `website/` and an ignored-build step so commits that touch only the Rust workspace do not redeploy.

Bun only: `package.json` sets `packageManager` and `engines` to Bun and no Node version, every script runs under Bun (`bun --bun`), and `vercel.json` sets `bunVersion` (the Bun runtime for Vercel Functions, in beta). Vercel installs with Bun because of `bun.lock`.
