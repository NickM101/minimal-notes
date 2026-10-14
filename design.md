# Zen Note — Design Specification & Implementation Plan

Based on the design system and UI flows documented in `inspo/` (Zen Note).

---

## 1. Design Philosophy & Aesthetic Core
- **Concept**: *Zen Note* — bringing the serenity, simplicity, and natural tactile experience of old books and paper into modern digital note taking. Monastic simplicity, deliberate whitespace, unadorned elegance.
- **Color Palette**:
  - **Old Paper (Background)**: `#DDDBC7` (or rgba equivalent `221, 219, 199`)
  - **Blackov (Ink / Foreground)**: `#000000` (or deep charcoal `#1A1A1A` for softer contrast where appropriate)
  - **Accent Highlight**: Yellow `#FFE600` / `#F8DF00` (used for active text marker / highlight tag, e.g. `<mark>` or active selection indicator)
  - **Muted Lines & Borders**: Subtle ink lines (`#000000` with 10%–20% opacity or hairline `#C9C6B0`)
- **Typography**:
  - **Headline / Serif / Slab**: *Zilla Slab* / Serif styling for headers, title branding, and note titles.
  - **Body**: Clean legible serif / slab or editorial sans with generous line-height (`1.5` - `1.6`) and letter-spacing reflecting physical book typesetting.
- **Icons**:
  - Fine hairline outline stroke icons (Checklist, Bullets, Ordered List, Bold `B`, Italic `I`, Underline `U`, Highlight `T`, Search, Play/Pause/Stop, Back `<`).

---

## 2. Information Architecture & Navigation Flow

According to the wireframes (`33a1a075021533.5c40da0a1b505.png`, `c6a26475021533.5c420877e9707.png`, `267e8275021533.5c420877e94f2.png`):

```
                   ┌─────────────────────────────┐
                   │         Home View           │
                   │   - "Zen Note" branding     │
                   │   - Zen lamp / art graphic  │
                   │   - [ New Note ] button     │
                   │   - "or"                    │
                   │   - [ Browse Notes ] link   │
                   │   - Search icon (top right) │
                   └──────────────┬──────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         ▼                        ▼                        ▼
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│   Browse Notes   │    │   Search Notes   │    │  Editor View     │
│ - Note list rows │    │ - Live search bar│    │ - Title & Date   │
│   (Title + Date) │    │ - Filtered list  │    │ - Note Body      │
│ - [ New Note ]   │    │ - Highlight query│    │ - Media preview  │
│ - Instant filter │    │                  │    │ - Voice player   │
└────────┬─────────┘    └─────────┬────────┘    │ - Bottom Toolbar │
         │                        │             └────────┬─────────┘
         └────────────────────────┴──────────────────────┘
                                  ▲
                                  │
                         ┌────────┴────────┐
                         │   Add Modal     │
                         │ - Add image     │
                         │ - Add voice rec │
                         └─────────────────┘
```

### Views:
1. **Home / Hero Screen**:
   - Top bar: `Zen Note` logo left, search button right.
   - Minimalist centerpiece hanging lamp graphic / zen silhouette with vertical Japanese-style tagline `"Be Minimalist"`.
   - Action buttons: Primary boxed `[ New Note ]`, and `Browse Notes` button.
2. **Browse / Search Screen**:
   - Clean search field with search glyph and clear button `X`.
   - List of notes rendered in pure Zen book index style: Title left-aligned, Date right-aligned, separated by delicate horizontal ruled lines.
   - Quick "New Note" footer button.
3. **Note Editor Screen**:
   - Top action bar: `< Back`, `Save`, `Add` (+ menu for media/voice).
   - Document canvas:
     - Editable Title with underline separator.
     - Note date indicator.
     - Checklist items (interactive checkboxes `[ ]` / `[x]`).
     - Rich content: Text paragraphs, highlighted text spans.
     - Attached gallery images inline.
     - Voice recording waveform/player bar (`00:00:00`, Progress bar, Play `▶`, Pause `⏸`, Delete `✕`).
   - Bottom formatting toolbar:
     - Checklist toggle (`☑`)
     - Bullet list (`≡`)
     - Numbered list (`1.`)
     - Bold (`B`)
     - Italic (`I`)
     - Underline (`U`)
     - Text Highlight (`T` in yellow square)
4. **Media & Voice Attachment Dialog / Panel**:
   - "Add image" (allows selecting image or choosing from sample zen gallery)
   - "Add voice recording" (interactive voice recording simulator with duration timer and audio bar)

---

## 3. Missing Features Identified from Inspo & Roadmap

1. **Zen Styling Overhaul**:
   - Implement the exact `#DDDBC7` old paper background and `#000000` ink aesthetic.
   - Refined serif/slab typography, line heights, and delicate editorial borders.
2. **View Architecture**:
   - Introduce Home (Cover), Note List / Search, and Note Editor modes with seamless fluid transitions.
3. **Interactive Checklists & Tasks**:
   - Support checklist items that can be checked/unchecked interactively.
4. **Formatting Toolbar**:
   - Bottom toolbar for Quick Formatting: Checklists, Bullet lists, Numbered lists, Bold, Italic, Underline, and Highlight marker.
5. **Image Attachment & Inline Gallery**:
   - Ability to add images to notes (stored in note metadata), rendering elegant framed images inline.
6. **Voice Recording Audio Player**:
   - Integrated audio recording / player bar with timestamp `00:00:00`, scrub bar, play/pause controls, and dismiss.
7. **Zen Ambient Illustrations**:
   - Minimalist pendant lamp illustration and zen graphic accents true to the Behance reference designs.
