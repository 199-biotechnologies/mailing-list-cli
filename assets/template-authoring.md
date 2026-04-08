## 16. Appendix: Embedded Template Authoring Guide

The full text of `template guidelines` (also in `assets/template-authoring.md`):

````markdown
# Template Authoring for mailing-list-cli

Read this once before authoring your first template. Two minutes here saves
hours of debugging Outlook desktop.

## The single most important rule

Use only `<mj-*>` tags. Never write raw `<table>`, `<div>`, `<style>`, or any
CSS that uses `flex`, `grid`, `float`, or `position`. The MJML compiler emits
the gnarly nested-table HTML that Outlook desktop needs. If you hand-write
HTML, your email will break in Outlook for 30% of recipients.

## Template structure

Every template has YAML frontmatter declaring its variable schema, followed by
MJML markup with Mustache merge tags.

```mjml
---
name: snake_case_name
subject: "Subject line with {{ first_name }}"
variables:
  - name: first_name
    type: string
    required: true
  - name: company
    type: string
    required: false
---
<mjml>
  <mj-head>
    <mj-title>Email title (shows in tab)</mj-title>
    <mj-preview>Preview text (shows in inbox list)</mj-preview>
  </mj-head>
  <mj-body>
    <mj-section>
      <mj-column>
        <mj-text>Hello {{ first_name }}</mj-text>
        <mj-button href="https://example.com">Click here</mj-button>
        {{{ unsubscribe_link }}}
        {{{ physical_address_footer }}}
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
```

## Required placeholders

Every template MUST include both of these in the body, or `template lint`
will refuse to save it:

- `{{{ unsubscribe_link }}}` — replaced at send time with a one-click
  unsubscribe link bound to the recipient
- `{{{ physical_address_footer }}}` — replaced with your CAN-SPAM physical
  address from `config.toml`

The triple braces are mandatory: they tell Handlebars not to HTML-escape the
output, because these placeholders inject HTML.

## MJML components you can use

- `<mj-section>` — a horizontal stripe; the top-level layout primitive
- `<mj-column>` — a vertical column inside a section; up to 4 per section
- `<mj-text>` — paragraph text
- `<mj-button>` — a button with `href`, `background-color`, etc.
- `<mj-image>` — an image with `src`, `alt`, `width`, etc.
- `<mj-divider>` — a horizontal line
- `<mj-spacer>` — vertical whitespace
- `<mj-social>` / `<mj-social-element>` — social media link rows
- `<mj-table>` — a real data table (if you genuinely need one)
- `<mj-raw>` — escape hatch for hand-written HTML; only use if you know
  exactly what you're doing

Every component supports `padding`, `background-color`, `color`, `font-size`,
`font-family`, and other standard email-safe attributes.

**Compatibility note:** `mailing-list-cli` compiles MJML with the Rust crate
`mrml 5.1`, NOT the reference JavaScript MJML implementation. Most standard
attributes work identically, but **uncommon or newer attributes may be silently
dropped** by mrml without a warning. Examples include things like
`border-radius` on `<mj-section>` (works in JS MJML, dropped in mrml 5.1).

If an attribute doesn't appear in the rendered output, that's the most likely
explanation. Always verify with `template render --with-placeholders` or send a
test via `broadcast preview <id> --to your@email.com` before relying on it.
When in doubt, stick to the attributes shown in the MJML docs for the core
components above — those are well-supported by mrml.

## Merge tags

Use Mustache syntax with snake_case variable names:

```
{{ first_name }}
{{ company }}
{{ unsubscribe_link }}     ← HTML-escaped (almost never what you want for links)
{{{ unsubscribe_link }}}   ← raw HTML (what you want for the unsubscribe link)
```

Conditionals:

```
{{#if company}}
  <mj-text>From {{ company }}</mj-text>
{{else}}
  <mj-text>From a friend</mj-text>
{{/if}}
```

Loops (`{{#each}}`) and partials (`{{> name}}`) are NOT supported. Templates
are intentionally single-file — if you need a shared section, duplicate it
across templates. If you need to generate many variants from one schema,
generate the templates programmatically with `template create --from-file`.

## Variables built into every template

You don't have to declare these — they're injected by the send pipeline:

- `first_name` (string, may be empty)
- `last_name` (string, may be empty)
- `email` (string, always present)
- `unsubscribe_link` (HTML, use triple braces)
- `physical_address_footer` (HTML, use triple braces)
- `current_year` (number)
- `broadcast_id` (number)

Any custom field you've created with `field create <key>` is also available as
`{{ key }}` if the recipient has a value for it.

## Common gotchas

1. **Don't link to `mailto:` for unsubscribe.** Use `{{{ unsubscribe_link }}}`.
   The send pipeline injects a real one-click unsubscribe URL.
2. **Don't omit the preview text.** It dramatically affects open rate.
3. **Don't put images larger than 600px wide.** Most email clients render at
   600px. Larger images get scaled, ugly.
4. **Don't use background images for important content.** Outlook desktop
   strips them.
5. **Don't write subject lines longer than 50 characters.** They get truncated
   on mobile.
6. **Don't forget the plain-text alternative.** It's auto-generated by the
   pipeline, but if your HTML is junk, the plain text will be too.

## Validation

Run `mailing-list-cli template lint <name>` before sending. It will catch
missing placeholders, broken merge tags, dangerous CSS, and gives you a
preview of the rendered HTML.

## Preview

Run `mailing-list-cli template render <name> --with-data sample.json` to
get the rendered HTML printed to stdout.

Run `mailing-list-cli broadcast preview <broadcast-id> --to your-test@email.com`
to send a real test through Resend.

## When in doubt

Read [mjml.io/try-it-live](https://mjml.io/try-it-live) — the official MJML
playground. Every component is documented there, but remember the compatibility
note above: `mailing-list-cli` uses `mrml 5.1` (Rust), not the JS MJML engine
the playground runs. Stick to core components and standard attributes, and
always verify the rendered output with `template render --with-placeholders`.
````
