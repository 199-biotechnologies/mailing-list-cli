# Email Templates for an Agent-First Rust CLI

Research brief for `mailing-list-cli`. Date: 2026-04-07.

This report answers: what template format should `mailing-list-cli` adopt so that LLM
agents can author templates that actually render across Gmail, Outlook desktop, Apple Mail,
and Yahoo, *and* the Rust binary can compile + validate them with no Node.js dependency?

The recommendation is up front in section 0. The rest is the evidence.

---

## 0. Recommendation in one paragraph

**Use MJML as the source format, parsed and rendered in-process by the [`mrml`](https://crates.io/crates/mrml)
Rust crate** (a pure-Rust reimplementation of MJML by Jérémie Drouet, ~3 ms render time, no
Node, no subprocess). Variables use **Mustache-style `{{ snake_case }}`** rendered by the
[`handlebars`](https://crates.io/crates/handlebars) crate, with a strict pre-flight schema
declared at the top of every template (`---` YAML frontmatter). Inline CSS at compile time
via [`css-inline`](https://crates.io/crates/css-inline). Generate the plain-text alternative
with [`html2text`](https://crates.io/crates/html2text). Validate offline against
[caniemail.com](https://www.caniemail.com/) data and a hand-curated rule set ("no flexbox",
"no background-image on divs", "<102 KB", etc.). Ship a 1-page `TEMPLATE-AUTHORING.md` that
the CLI prints on `template create --help` so the agent has the rules in-context before it
writes a single line.

This combination is **agent-friendly** (MJML is verbose-but-regular, Mustache is the lowest
friction merge syntax), **Rust-native** (no Node, single static binary), and **opinionated**
(the framework absorbs the Outlook/Gmail/Apple Mail compatibility burden so the LLM doesn't
have to know that flexbox is forbidden).

---

## 1. Template format landscape

### 1.1 MJML (Mailjet Markup Language)

MJML is a markup language created at Mailjet in 2015. You write semantic tags (`<mj-section>`,
`<mj-column>`, `<mj-button>`, `<mj-image>`) and the compiler emits the gnarly nested-table
HTML with inline CSS that actually renders in Outlook 2016. It is the de facto standard for
hand-authored responsive HTML email outside the React ecosystem
([mjml.io](https://mjml.io)).

**Pros for an agent-first CLI**:
- The vocabulary is small and regular. An LLM trained on MJML produces valid output reliably,
  because there are only ~30 component tags and they all follow the same `mj-*` prefix.
- The compiler handles every Outlook/Gmail/Apple Mail bug. The agent does not need to know
  about VML, conditional comments, or the Word rendering engine — those are MJML's problem.
- It is **fully implementable in Rust** via [`mrml`](https://github.com/jdrouet/mrml). Quote
  from the project: "less than 1.7 MB at startup… ~174x faster than Node MJML for complex
  templates" ([mrml on crates.io](https://crates.io/crates/mrml)). `mrml-cli` is also
  published on crates.io.
- MJML supports `<mj-include>` for partials, so a template library can share headers and
  footers ([mrml docs](https://docs.rs/mrml)).
- Output is deterministic and stable across releases.

**Cons**:
- Verbose. A simple email is ~30 lines of MJML vs. ~5 lines of markdown.
- `<mj-style[inline]>` is **not yet implemented in `mrml`**, so you need a separate inlining
  step (`css-inline` crate handles that).
- Custom components are a JS thing — `mrml` only ships the core component set.

**Verdict**: This is the right answer for `mailing-list-cli`. Native Rust implementation
exists, agent-friendly grammar, absorbs the cross-client rendering hell.

### 1.2 React Email (`react.email`)

Resend's own framework. JSX components like `<Button>`, `<Heading>`, `<Text>` that compile
to email-safe HTML via `@react-email/render` ([react.email](https://react.email)).

**Pros**:
- Maintained by Resend, so the integration story with the Resend send API is smoothest.
- Excellent component ecosystem and a real preview server.
- Type-safe templates if you write TypeScript.

**Cons**:
- **Hard requirement on Node.js**. The CLI binary uses `vm.SourceTextModule` and forks itself
  with `--experimental-vm-modules`
  ([deepwiki](https://deepwiki.com/resend/react-email/3.1-react-email-cli)). There is no Rust
  port and there will not be one — it is React under the hood.
- An LLM authoring React Email templates needs to know JSX, the React Email component API,
  *and* email rendering rules. That is more cognitive surface than MJML.
- Templates are code, not data. They are harder to store in SQLite as text and harder to
  diff/version.
- Invalidates the "single static Rust binary" goal of the CLI.

**Verdict**: Excellent for Node teams that own React. **Disqualified** for a Rust-only CLI.

### 1.3 jsx-email (`jsx.email`)

Community alternative to React Email; same JSX-based authoring model but separate
maintainers and a slightly different component set ([jsx.email](https://jsx.email)). Same
fundamental dependency on Node + React. Same disqualification.

### 1.4 mjml-react

Authoring MJML *as JSX components*, then compiling to MJML, then to HTML. Two compile steps,
two runtime dependencies, no win for an agent-first CLI.

### 1.5 Maizzle (`maizzle.com`)

Tailwind-for-email. You write HTML with Tailwind utility classes; the build pipeline inlines
CSS, removes unused classes, and minifies ([maizzle.com](https://maizzle.com)). Modern, fast,
liked by hand-coding email designers.

**Pros**:
- Tailwind is one of the most LLM-fluent CSS dialects on Earth — agents write it well.
- Output is just HTML; no exotic tags.

**Cons**:
- **Node-only** build pipeline. No Rust port.
- Tailwind does not magically fix Outlook. You still need to author email-safe HTML
  (tables, no flex). Tailwind helps with the *styling* but not the *structure*. The agent
  still needs to know the bad-Outlook rules.
- The Maizzle build does a lot of post-processing the agent cannot anticipate at author
  time.

**Verdict**: Strong second place if Node were on the table. It is not.

### 1.6 Foundation for Emails (Zurb / Inky)

Zurb's framework, abandoned by its original maintainers but still alive on GitHub
([foundation/foundation-emails](https://github.com/foundation/foundation-emails)). The Inky
templating language (`<row>`, `<columns>`) is conceptually similar to MJML but has fewer
components and a smaller community in 2026.

**Verdict**: Strictly dominated by MJML. No Rust port. Skip.

### 1.7 Plain HTML with inlined CSS

Lowest common denominator. Author table-based HTML by hand, run it through `css-inline`,
ship.

**Pros**:
- Zero abstraction, zero compile step, zero magic. Always works.
- Easiest possible thing to store and version.

**Cons**:
- An LLM authoring raw email HTML will produce something that breaks in Outlook ~50 % of the
  time. The grammar is too irregular and the gotchas are too client-specific. Without the
  abstraction layer, every template is a Russian-roulette of bugs.
- The CLI gets no leverage — it cannot offer the agent "you can use `<mj-button>` and we'll
  handle the rest".

**Verdict**: Not the primary format. But the CLI should *accept* raw HTML as an "expert
escape hatch" for templates a designer hand-built and validated externally.

### 1.8 Markdown-first (Buttondown-style)

Author in Markdown, the platform converts to HTML at send time. This is what Buttondown,
Substack, and most newsletter tools use for the actual writing experience.

**Pros**:
- LLMs are *exceptional* at Markdown.
- Tiny source files, easy to version, easy to read.
- Renders perfectly in plain-text fallback (it already is plain text).

**Cons**:
- The HTML conversion still needs to produce email-safe markup. You still need a "shell"
  template (header, footer, button styling) that the markdown body lives inside.
- No native support for complex layouts (multi-column, hero images with overlays).

**Verdict**: This is what the agent will *want* to write 90 % of the time. The CLI should
support a hybrid: **MJML shell + Markdown body**. Author writes Markdown, CLI wraps it in a
chosen MJML shell (`shell-default.mjml`, `shell-product-launch.mjml`, etc.), MJML compiles
the lot to HTML. The agent writes 12 lines of Markdown; the CLI emits 800 lines of nested
table HTML.

### 1.9 Format scoring matrix

| Format | Agent-friendly | No Node | Cross-client safety | Stable output | Verdict |
|---|---|---|---|---|---|
| **MJML (via `mrml`)** | High | Yes | Excellent | Excellent | **Primary** |
| **MJML shell + Markdown body** | Highest | Yes | Excellent | Excellent | **Default authoring path** |
| React Email | Medium | **No** | Excellent | Good | Disqualified |
| jsx-email | Medium | **No** | Excellent | Good | Disqualified |
| Maizzle | High | **No** | Good | Good | Disqualified |
| Foundation/Inky | Medium | **No** | Good | Good | Skip |
| Raw HTML + inline CSS | **Low** | Yes | Manual | Excellent | Escape hatch |
| Markdown-only (no shell) | Highest | Yes | Poor (no shell) | Excellent | Insufficient alone |

---

## 2. What breaks emails in real clients (the gotcha list)

Sources: [Litmus rendering guide](https://www.litmus.com/blog/a-guide-to-rendering-differences-in-microsoft-outlook-clients),
[Email on Acid](https://www.emailonacid.com/blog/article/email-development/how-to-code-emails-for-outlook/),
[Mailtrap rendering issues](https://mailtrap.io/blog/email-rendering-issues-outlook/),
[caniemail.com](https://www.caniemail.com/).

The historical context: Outlook 2007-2024 on Windows uses **Microsoft Word's HTML rendering
engine**, which is a 1990s subset of HTML/CSS plus VML. Microsoft has announced the new
Outlook (Edge/WebView2-based) will fully replace the Word engine in **October 2026**, but
until that switchover lands and propagates, every email author must still target the Word
engine. Gmail strips `<style>` blocks aggressively. Apple Mail is generous (WebKit). Yahoo
is somewhere in the middle.

### Top 10 things that will break your email

1. **Flexbox and CSS Grid in Outlook desktop.** Word engine ignores both completely. Layouts
   collapse to a single column or stack unpredictably. **Fix**: tables only.
2. **`background-image` on `<div>` or non-`<body>` elements in Outlook.** Silently dropped.
   **Fix**: VML rect with `mso-` conditional comments, or just use `<table background>` on
   the body.
3. **Margins on `<div>` and `<img>` in Outlook.** Stripped. **Fix**: padding on parent table
   cell, or empty spacer cells.
4. **Float, position, and clear in Outlook.** Ignored. **Fix**: nested tables.
5. **CSS `border-radius` (rounded buttons) in Outlook.** Ignored — buttons render as sharp
   rectangles. **Fix**: VML "bulletproof button" or accept square corners as a graceful
   degradation.
6. **`<style>` block in Gmail (specifically the Gmail iOS app and Gmail webmail without a
   custom domain).** Gmail strips media queries from `<style>` in some configurations, and
   non-Google addresses (Yahoo, etc.) on Gmail Web have historically dropped `<style>`
   entirely. **Fix**: inline every critical style. Keep a small `<style>` for media queries
   only, but never depend on it for the desktop layout.
7. **102 KB Gmail clip.** Gmail truncates messages above ~102 KB of HTML and shows "[Message
   clipped]". The clipped portion may break the unsubscribe link or the tracking pixel.
   **Fix**: budget. Strip comments, minify, prefer shared `mj-include` partials over
   duplication.
8. **Unsupported web fonts.** Most clients fall back to system fonts. **Fix**: declare a
   `font-family` stack with `Arial, Helvetica, sans-serif` as the floor. Web fonts via
   `<link>` only work in Apple Mail and a few others.
9. **Dark mode color inversion.** Gmail, Apple Mail, and Outlook each have their own
   inversion algorithm. White-on-light-gray text becomes invisible-on-dark. **Fix**: avoid
   pale grays for body text; use a `meta name="color-scheme" content="light dark"` and
   `prefers-color-scheme` media queries; test with images that have transparent backgrounds.
10. **Missing or malformed plain-text alternative.** Spam filters down-rank HTML-only mail,
    treating it as a "hashbusting" pattern. **Fix**: always send `multipart/alternative`
    with a coherent text version (see §5).

Honorable mentions: dropped `<form>` elements (most clients strip), `<video>` only works in
Apple Mail and a few others, CSS animations only in Apple Mail, `position: fixed` on
nothing, JavaScript on nothing (always and forever blocked).

The MJML compiler eliminates 1-7 automatically. 8-10 are still the author's problem, which
is why the agent guidelines doc must call them out explicitly.

---

## 3. Variable substitution / merge tags

The merge-tag question is really two questions: *what does the wire protocol look like*, and
*what does the author write in the source template*.

### 3.1 Survey of conventions

| Platform | Author syntax | Conditionals | Loops |
|---|---|---|---|
| Mustache | `{{name}}` | `{{#subscribed}}…{{/subscribed}}` | `{{#items}}…{{/items}}` |
| Handlebars | `{{name}}`, `{{{html}}}` for raw | `{{#if subscribed}}…{{/if}}` | `{{#each items}}…{{/each}}` |
| Liquid (Shopify, Jekyll) | `{{ name }}` | `{% if subscribed %}…{% endif %}` | `{% for item in items %}…{% endfor %}` |
| Mailchimp merge tags | `*\|FNAME\|*` | `*\|IF:SUBSCRIBED\|*…*\|END:IF\|*` | n/a |
| Mailgun / Sendgrid (legacy) | `%recipient_first_name%` | n/a | n/a |
| Resend | **`{{{NAME}}}`** (triple-brace, uppercase) | n/a (server-side substitution only) | n/a |

Resend's choice is documented at [Resend template variables](https://resend.com/docs/dashboard/templates/template-variables).
The Resend conventions:
- Triple curly braces: `{{{NAME}}}`
- ASCII letters, digits, underscore, max 50 chars
- Recommended uppercase (`{{{PRODUCT}}}`)
- Reserved keys: `FIRST_NAME`, `LAST_NAME`, `EMAIL`, `RESEND_UNSUBSCRIBE_URL`, `contact`,
  `this`
- Variables resolve via Resend's contacts table; passing extra `variables` in the send call
  injects ad-hoc values
- No conditionals or loops on Resend's side

### 3.2 Which is best for an LLM author?

**Mustache `{{ snake_case }}`** is the clearest winner for LLM authoring. Reasons:
- It is the most-trained-on syntax in language model corpora (Mustache shipped in 2009;
  Handlebars 2010; both are everywhere on GitHub).
- It is visually obvious, hard to typo, and fails loudly (an unrendered `{{name}}` is
  immediately recognizable as a leaked variable).
- `*|FNAME|*` and `%FNAME%` look like other ASCII art and are typo magnets.
- Triple-brace `{{{X}}}` is a Handlebars convention meaning "do not HTML-escape" — it is
  *reasonable* for Resend's use case (server already escapes appropriately) but it is **less
  agent-friendly** than the standard double-brace because LLMs tend to write `{{X}}` by
  default.

### 3.3 Recommended scheme for `mailing-list-cli`

Author in Mustache `{{ snake_case }}` style. Render via the
[`handlebars`](https://crates.io/crates/handlebars) Rust crate at `template build` time
(before send). At the wire layer, the CLI emits **fully-rendered HTML** to Resend's `send`
endpoint — *not* a Resend template. Reasons:

1. The CLI controls the rendering, so it controls the variable syntax. The agent never has
   to think about Resend's `{{{TRIPLE_BRACE}}}` quirk.
2. The CLI can offer real conditionals (`{{#if user.is_paid}}`) and loops
   (`{{#each products}}`) that Resend's server-side templates cannot.
3. Validation runs locally before any API call.
4. The same template format works if the user later swaps Resend for Postmark / SES.

The `mailing-list-cli` glossary becomes:
- `template create <name>` — author a new MJML+Markdown template
- `template get <name>` — fetch source for an agent to read
- `template lint <name>` — validate offline
- `template build <name> --vars vars.json` — render to HTML + plain text
- `template preview <name> --vars vars.json` — render and open in browser
- `template send <name> --to … --vars …` — render and call Resend

### 3.4 Conditionals and loops

LLMs handle Handlebars block helpers comfortably:
```
{{#if subscriber.is_paid}}
  <mj-button href="{{paid_dashboard_url}}">Open dashboard</mj-button>
{{else}}
  <mj-button href="{{upgrade_url}}">Upgrade</mj-button>
{{/if}}
```
The `handlebars` Rust crate supports `if`, `each`, `with`, `unless`, partials, and custom
helpers. Custom helpers should be kept *minimal* — a `{{format_date}}` and a
`{{format_currency}}` are enough; resist the temptation to invent a DSL.

---

## 4. Validation

### 4.1 What can run offline from a Rust CLI

| Tool | Offline? | Rust-native? | What it catches |
|---|---|---|---|
| `mrml` parse step | Yes | **Yes** | Invalid MJML structure, unknown components |
| [`html-validate`](https://html-validate.org/) | Yes | No (Node) | HTML5 validity, missing alt, unclosed tags |
| `caniemail.com` JSON dataset | Yes | No (data only — embed in Rust) | Per-feature client support warnings |
| Custom Rust linter using [`scraper`](https://crates.io/crates/scraper) crate | Yes | **Yes** | Email-specific gotchas (no flexbox, no `position`, no `<form>`, etc.) |
| `css-inline` | Yes | **Yes** | Catches malformed CSS during inlining |
| `html2text` | Yes | **Yes** | Confirms text fallback renders something coherent |
| MJML built-in validator | Yes | **Yes** (via mrml) | Schema validation of `mj-*` tags |
| Litmus / Email on Acid screenshot tests | **No** (paid SaaS) | n/a | Real client renders |
| Mailtrap | **No** (SaaS, has free tier) | n/a | Inbox preview, link checking |
| Gmail clip detection | Yes (file size check >102 KB) | **Yes** (just `wc -c`) | Gmail truncation |

### 4.2 Recommended validation pipeline (all offline, all Rust)

The CLI runs these on every `template build`:

1. **Parse with `mrml`** — fail on any unknown component or malformed structure.
2. **Render to HTML in-process** — fail on inlining errors via `css-inline`.
3. **Custom email-safe linter** — a small Rust module that walks the HTML with `scraper`
   and rejects:
   - `display: flex`, `display: grid`
   - `position: absolute|fixed|sticky`
   - `<form>`, `<script>`, `<iframe>`, `<object>`
   - `background-image` on non-`<body>` elements (warning, not error)
   - Missing `alt=""` on `<img>`
   - Anchor tags without `href`
   - Web font `<link>` without a fallback `font-family` stack
4. **Size check** — total HTML must be <90 KB to leave headroom under Gmail's 102 KB clip.
5. **Plain-text generation** — use `html2text`, then assert the output is non-empty and
   contains every `href` from the HTML.
6. **Variable schema check** — the YAML frontmatter declares the required variables; the
   linter verifies every `{{ var }}` in the body is declared, and every declared variable
   is used somewhere.
7. **Optional spam-text heuristic** — scan for ALL CAPS subjects, `$$$$`, "FREE", "Click
   here NOW", etc. Warning only.

**External SaaS** (Litmus, Email on Acid, Mailtrap inbox previews) should be an opt-in step
the user runs manually before a campaign goes out — *not* a precondition for build. Document
how to wire them up but do not depend on them.

---

## 5. Plain-text alternative

### 5.1 Why it is mandatory

- **Deliverability**: Sending HTML-only mail looks like a hashbusting spam pattern. Major
  spam filters (Microsoft especially) down-rank HTML-only messages
  ([Suped](https://www.suped.com/knowledge/email-deliverability/basics/does-plain-text-email-version-affect-deliverability),
  [Mailflow Authority](https://mailflowauthority.com/email-content/plain-text-vs-html-email)).
- **Accessibility**: Some screen readers and text-only clients (Mutt, Emacs gnus) only see
  the text part.
- **Bandwidth-constrained recipients**: Apple Watch, dumb phones, low-bandwidth contexts.

The fix is to send `multipart/alternative` with both. Resend handles this natively — pass
both `html` and `text` fields in the send payload.

### 5.2 Auto-generation in Rust

[`html2text`](https://crates.io/crates/html2text) (Chris Emerson) is the right tool. It
parses HTML with `html5ever` (Servo's parser), walks the DOM, and emits formatted text. It
preserves links as `[text](url)` style annotations and wraps to a configurable column width.

The pipeline:
1. Render MJML → HTML.
2. Run the HTML through `html2text` with width=72.
3. Post-process: collapse runs of blank lines, ensure `View in browser:` and unsubscribe
   URLs are inlined.
4. Sanity-check: text must contain at least one URL if the HTML did, and must not be empty.

The agent should *also* be able to author a hand-written `text` block in the template
frontmatter if they want full control — auto-generation is the default, manual override is
the escape hatch.

### 5.3 What auto-generation gets wrong

[Litmus](https://www.litmus.com/blog/best-practices-for-plain-text-emails-a-look-at-why-theyre-important)
warns that some auto-generators turn `<strong>` into ALL CAPS, which spam filters flag.
`html2text` does *not* do this by default — it just drops the markup. Verified safe.

---

## 6. Agent author guidelines (the in-CLI README)

This is the document the CLI prints when an agent runs `template create --help` or when the
agent first opens a new template file. It is intentionally short. It is written *to* an LLM
agent, not to a human designer.

The draft is in the next section.

---

## 7. `TEMPLATE-AUTHORING.md` — draft for inclusion in the CLI

```markdown
# How to write a mailing-list-cli template

You are an AI agent authoring an email template. Read this entire document before writing
any code. The CLI will reject templates that violate the rules below — fix them locally,
do not assume the linter is wrong.

## The single most important rule

**Use only the `<mj-*>` components listed in §4. Never write a raw `<table>`, `<div>`,
`<style>`, `<script>`, `<form>`, `<iframe>`, or any HTML element that uses `display: flex`,
`display: grid`, `position: absolute`, `position: fixed`, or `float`.** The MJML compiler
turns `<mj-section>`/`<mj-column>` into the correct nested-table HTML for you. If you
hand-write tables, your email will break in Outlook desktop. There are no exceptions.

## File structure

A template is a single `.mjml.md` file with YAML frontmatter:

    ---
    name: welcome
    subject: "Welcome to {{ company_name }}, {{ first_name }}!"
    preheader: "Here's how to get started in 60 seconds."
    from: "Boris <boris@199.bio>"
    variables:
      first_name: { type: string, required: true, example: "Alex" }
      company_name: { type: string, required: true, example: "199 Biotech" }
      cta_url: { type: string, required: true, example: "https://199.bio/start" }
      is_paid_subscriber: { type: boolean, required: false, default: false }
    ---

    <mjml>
      <mj-head>
        <mj-title>Welcome to {{ company_name }}</mj-title>
        <mj-preview>{{ preheader }}</mj-preview>
        <mj-attributes>
          <mj-all font-family="Arial, Helvetica, sans-serif" />
          <mj-text font-size="16px" line-height="24px" color="#1a1a1a" />
        </mj-attributes>
      </mj-head>
      <mj-body background-color="#f6f6f6">
        <mj-section background-color="#ffffff" padding="32px">
          <mj-column>
            <mj-text font-size="22px" font-weight="bold">
              Hi {{ first_name }},
            </mj-text>
            <mj-text>
              Welcome to {{ company_name }}. You're in.
            </mj-text>
            <mj-button href="{{ cta_url }}" background-color="#1a1a1a">
              Get started
            </mj-button>
            {{#if is_paid_subscriber}}
            <mj-text font-size="14px" color="#666666">
              You're on the paid plan. Thank you.
            </mj-text>
            {{/if}}
          </mj-column>
        </mj-section>
      </mj-body>
    </mjml>

## Variables

- Syntax: `{{ snake_case_name }}`. Lowercase, underscores, never spaces.
- Every variable used in the body **must** be declared in frontmatter `variables:`.
- Every declared variable **must** be used in the body or subject.
- The CLI rejects templates that have undeclared or unused variables.
- Conditionals: `{{#if var}}…{{else}}…{{/if}}`. `unless` works too.
- Loops: `{{#each items}}…{{this.field}}…{{/each}}`.
- Reserved variables (always available, do not declare):
  - `unsubscribe_url` — generated per recipient by the CLI
  - `view_in_browser_url` — generated per recipient
  - `current_year` — current year as integer
- Helpers: `{{ format_date date "YYYY-MM-DD" }}`, `{{ format_currency cents "USD" }}`.

## Allowed MJML components (the only ones the linter accepts)

| Component | Use for |
|---|---|
| `mj-body` | Outer wrapper, set `background-color` |
| `mj-section` | Horizontal row, full width |
| `mj-column` | Vertical column inside a section |
| `mj-text` | Any paragraph, heading, or rich text |
| `mj-button` | Call-to-action — never use an `<a>` styled as a button |
| `mj-image` | Any image — always set `alt`, `width`, and `href` if linked |
| `mj-divider` | Horizontal rule |
| `mj-spacer` | Vertical spacing |
| `mj-table` | Tabular data only (e.g. order summary) |
| `mj-social` | Social media link row |
| `mj-navbar`, `mj-navbar-link` | Top nav |
| `mj-include` | Pull in shared partials (`header.mjml`, `footer.mjml`) |

## Hard rules (the linter enforces these)

1. **Total HTML output must be ≤ 90 KB.** Gmail clips at ~102 KB. Strip whitespace, share
   partials, do not embed base64 images.
2. **Every `<mj-image>` needs `alt`, `width`, and (if linked) `href`.** No exceptions.
3. **Every `<a>` produced by `<mj-button>` needs an `href`.** Never `href="#"`.
4. **Use the system font stack: `Arial, Helvetica, sans-serif`** as the fallback floor. Web
   fonts can be added via `<mj-font>` but the fallback must always include a system font.
5. **Use a 600px maximum body width** (the MJML default). Do not override unless you have a
   reason.
6. **No `<style>` block with critical layout CSS.** Inline everything via `mj-attributes`.
   `<mj-style>` is fine for media queries (dark mode, mobile font sizing).
7. **Subject line ≤ 70 characters.** Anything longer truncates in Gmail mobile.
8. **Preheader text 40-130 characters.** Required.
9. **Image color contrast must work in dark mode** — never pure white logo on transparent
   background. Use `prefers-color-scheme` media query for swap.
10. **Plain-text alternative is auto-generated.** You may override by adding a `text:`
    field to frontmatter — write it as if for an old-school text-only email client. No
    Markdown, no HTML, just paragraphs and URLs spelled out.

## Things that will get your template rejected

- Any `<table>`, `<tr>`, `<td>` (use `mj-section`/`mj-column`)
- Any `<style>` block with `display`, `position`, `float`, `flex`, or `grid`
- Any `<script>`, `<form>`, `<iframe>`, `<object>`, `<embed>`
- Any `background-image` style on a `<div>` or `<mj-column>` (use `mj-section
  background-url=""`)
- Any URL shortener (`bit.ly`, `tinyurl`, `t.co`) — spam filters penalize these
- Subject lines in ALL CAPS, with multiple `!`, or containing the word "FREE"
- An `<mj-button>` whose label is just an emoji or just "→"

## Things that are fine but discouraged

- Custom fonts via `<mj-font>` (most clients fall back; Outlook desktop ignores them)
- Animated GIFs (work in Apple Mail and Gmail but not Outlook desktop, which shows the
  first frame)
- `border-radius` on buttons (silently ignored by Outlook desktop — buttons render square
  there, which is fine)
- Dark-mode-specific images (only Apple Mail and some Gmail builds honor them)

## Workflow

1. `template create welcome` — scaffolds a new file with the frontmatter and an empty body
2. Author the template
3. `template lint welcome` — runs all offline checks
4. `template build welcome --vars vars.json` — emits HTML + text
5. `template preview welcome --vars vars.json` — opens rendered HTML in browser
6. `template send welcome --to test@example.com --vars vars.json` — sends one test email

If `lint` fails, **fix the template — do not retry the lint hoping for a different
result**. The linter is deterministic.

## When in doubt

Default to: a single `mj-section`, single `mj-column`, `mj-text` for prose, one
`mj-button` for the CTA, one `mj-image` for the hero. This renders perfectly everywhere
and converts as well as anything more elaborate.
```

---

## 8. Sources

- [MJML — mjml.io](https://mjml.io)
- [mrml on GitHub](https://github.com/jdrouet/mrml)
- [mrml on crates.io](https://crates.io/crates/mrml)
- [mrml-cli on crates.io](https://crates.io/crates/mrml-cli)
- [React Email — react.email](https://react.email)
- [React Email CLI — DeepWiki](https://deepwiki.com/resend/react-email/3.1-react-email-cli)
- [jsx-email](https://jsx.email)
- [Maizzle](https://maizzle.com)
- [Foundation for Emails](https://github.com/foundation/foundation-emails)
- [Resend send-email API](https://resend.com/docs/api-reference/emails/send-email)
- [Resend template variables](https://resend.com/docs/dashboard/templates/template-variables)
- [Resend introducing templates](https://resend.com/blog/introducing-templates)
- [Litmus — Outlook rendering guide](https://www.litmus.com/blog/a-guide-to-rendering-differences-in-microsoft-outlook-clients)
- [Email on Acid — coding for Outlook](https://www.emailonacid.com/blog/article/email-development/how-to-code-emails-for-outlook/)
- [Mailtrap — Outlook rendering issues](https://mailtrap.io/blog/email-rendering-issues-outlook/)
- [Designing High-Performance Email Layouts in 2026](https://medium.com/@romualdo.bugai/designing-high-performance-email-layouts-in-2026-a-practical-guide-from-the-trenches-a3e7e4535692)
- [Email Mavlers — HTML email best practices](https://www.emailmavlers.com/blog/html-email-template-best-practices/)
- [caniemail.com](https://www.caniemail.com/)
- [caniemail scoreboard](https://www.caniemail.com/scoreboard/)
- [html-validate](https://html-validate.org/)
- [css-inline crate](https://crates.io/crates/css-inline)
- [html2text crate](https://crates.io/crates/html2text)
- [handlebars Rust crate](https://crates.io/crates/handlebars)
- [Suped — plain text deliverability](https://www.suped.com/knowledge/email-deliverability/basics/does-plain-text-email-version-affect-deliverability)
- [Litmus — best practices for plain text emails](https://www.litmus.com/blog/best-practices-for-plain-text-emails-a-look-at-why-theyre-important)
- [Mailflow Authority — plain text vs HTML deliverability](https://mailflowauthority.com/email-content/plain-text-vs-html-email)
- [Handlebars.js](https://handlebarsjs.com/)
- [Express templating cheatsheet](https://dev.to/alexmercedcoder/express-templating-cheatsheet-pug-ejs-handlebars-mustache-liquid-50f1)
- [gomjml — MJML for Go](https://preslav.me/2025/08/12/introducing-gomjml/)
</content>
</invoke>