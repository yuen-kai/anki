# Speedrun MCAT Ch31-35 — Concept-Summary Evaluator-Optimizer Report

Deck data: `rslib/src/speedrun/seed_data/mcat_ch31_35.json` (+ media in
`mcat_ch31_35_media/`). Concept **descriptions** are original-wording summaries;
concept **titles** are verbatim section/sub-section headings from the 2021-2022
Princeton Review MCAT Prep book; problems are extracted from the 2024 MCAT
Workout (verbatim stems/figures carried as image crops).

## Method

Evaluator-optimizer loop (rubric-based LLM-as-judge, per `agentic-eval`). Each of
the 20 concept summaries was scored against its **source** — the section's
heading + text extracted from the Prep PDF (the same region captured in the
concept's `-src.png` crop, plus the surrounding section paragraphs) — on four
dimensions:

- **faithfulness** — every claim supported by the source
- **accuracy** — no contradictions or fabrications
- **coverage** — captures the heading's core idea
- **concision** — 2-4 sentences, informationally dense

`overall = 0.35*faithfulness + 0.30*accuracy + 0.20*coverage + 0.15*concision`.
Threshold **0.85**; if below, refine and re-score (max 3 iterations, stop if not
improving).

## Per-concept results

| Concept (title)                                                  | faith     | acc       | cov       | conc | overall         | iters |
| ---------------------------------------------------------------- | --------- | --------- | --------- | ---- | --------------- | ----- |
| ch31-c1 The SN2 Mechanism                                        | 0.90      | 0.97      | 0.95      | 0.95 | **0.93**        | 1     |
| ch31-c2 Acidity and Enolization                                  | 0.85→0.94 | 0.95→0.97 | 0.85→0.95 | 0.95 | 0.88 → **0.94** | 2     |
| ch31-c3 Nucleophilic Addition Reactions to Aldehydes and Ketones | 0.95      | 0.97      | 0.95      | 0.95 | **0.95**        | 1     |
| ch31-c4 Esterification Reactions                                 | 0.90      | 0.95      | 0.95      | 0.90 | **0.92**        | 1     |
| ch32-c1 Amino Acid Structure and Nomenclature                    | 0.87      | 0.95      | 0.95      | 0.95 | **0.91**        | 1     |
| ch32-c2 The Peptide Bond                                         | 0.86      | 0.95      | 0.95      | 0.95 | **0.91**        | 1     |
| ch32-c3 Structure and Nomenclature of Monosaccharides            | 0.86      | 0.95      | 0.92      | 0.95 | **0.90**        | 1     |
| ch32-c4 Structure and Nomenclature of Disaccharides              | 0.90      | 0.95      | 0.95      | 0.95 | **0.93**        | 1     |
| ch33-c1 Kinematics                                               | 0.92      | 0.97      | 0.95      | 0.95 | **0.94**        | 1     |
| ch33-c2 Mass, Force, and Newton's Laws                           | 0.88      | 0.97      | 0.95      | 0.92 | **0.92**        | 1     |
| ch33-c3 Contact Forces: The Normal Force, Friction, and Tension  | 0.90      | 0.97      | 0.95      | 0.92 | **0.93**        | 1     |
| ch33-c4 Center of Mass, Torque, and Equilibrium                  | 0.87      | 0.97      | 0.95      | 0.95 | **0.92**        | 1     |
| ch34-c1 Work                                                     | 0.95      | 0.97      | 0.95      | 0.95 | **0.95**        | 1     |
| ch34-c2 Kinetic Energy                                           | 0.95      | 0.97      | 0.95      | 0.95 | **0.95**        | 1     |
| ch34-c3 Potential Energy                                         | 0.92      | 0.95      | 0.95      | 0.95 | **0.94**        | 1     |
| ch34-c4 Total Mechanical Energy                                  | 0.95      | 0.97      | 0.95      | 0.95 | **0.95**        | 1     |
| ch35-c1 Systems, Thermal Physics, and Thermodynamics             | 0.90      | 0.95      | 0.95      | 0.90 | **0.92**        | 1     |
| ch35-c2 The Zeroth Law of Thermodynamics                         | 0.92      | 0.97      | 0.95      | 0.95 | **0.94**        | 1     |
| ch35-c3 The First Law of Thermodynamics                          | 0.90      | 0.95      | 0.95      | 0.90 | **0.92**        | 1     |
| ch35-c4 The Second Law of Thermodynamics                         | 0.88      | 0.97      | 0.95      | 0.90 | **0.92**        | 1     |

**All 20 concepts pass the 0.85 threshold.** Mean final overall = **0.93**.

## Refinements applied

- **ch31-c2 Acidity and Enolization** (0.88 → 0.94, iteration 2): the draft drifted
  into keto-enol tautomerism (a neighboring sub-section) and omitted the source's
  central point. Rewrote to ground every claim in the "Acidity and Enolization"
  source: a strong base removes an alpha-proton to give the resonance-stabilized
  **enolate** (charge delocalized onto the carbonyl oxygen), which is **nucleophilic
  mainly at the alpha-carbon**; the planar alpha-carbon allows racemization, and a
  proton between two carbonyls is especially acidic.

## Notes on scoring

Where faithfulness sat at 0.86-0.90 on iteration 1, the summary stated correct,
standard facts about the section topic that appear later in the same numbered
section rather than in the first extracted paragraph (e.g., Newton's three laws
under "Mass, Force, and Newton's Laws"; hemiacetal/anomeric cyclization under
"...Monosaccharides"). These were judged faithful to the **section** (which the
concept title names) and left unchanged, as they added coverage without
introducing anything unsupported or contradictory.

---

# Audit & problem-source correction

## Trigger

A prior pass **authored 5 original Thermodynamics MCQs** (`ch35-c1-p4`,
`ch35-c2-p4`, `ch35-c3-p4`, `ch35-c4-p3`, `ch35-c4-p4`) to reach 16 problems for
the Thermodynamics leaf. Authoring violates the hard rule that every problem must
be a real, word-for-word question from the books. This section documents the audit
and the replacement of all 5 with real book questions.

## Audit of all 80 problems

Every problem's source was re-checked. Exactly **5 problems lacked an image crop**
— the 5 authored ones above — and no others. The remaining **75 are real 2024
Workout questions** (freestanding + practice-passage), each carried as a verbatim
`-stem.png` crop from the Workout PDF with `correctIndex` from that chapter's
printed SOLUTIONS key. No other authored, invented, or worked-example-turned-MCQ
problems were found (Ch31-34, including the "topped-up" physics chapters, draw only
on real Workout freestanding + practice-passage MCQs). **Post-fix: 0 authored
problems; all 80 have real book image crops.**

## Real thermodynamics 4-choice MCQ availability (recount)

- **Workout 2024, Ch35 Thermodynamics** (PDF pp. 427-430): **6 freestanding** +
  **5 practice-passage** (Thermoregulation) = **11** real 4-choice MCQs with a
  SOLUTIONS key.
- **Prep 2021-2022, Ch39 Thermodynamics** (PDF pp. 957-971): 7 worked "Examples"
  (39-1 … 39-7). **Correction to the stated assumption:** 6 of these (**39-2, 39-3,
  39-4, 39-5, 39-6, 39-7**) ARE genuine 4-choice (A-D) MCQs with a printed answer in
  the Solution; only **39-1** (a conduction-rate equation walk-through) is a
  non-MCQ. So Prep Ch39 supplies **6** real 4-choice MCQs with answer keys.
- **Total real thermo 4-choice MCQs across both books = 11 + 6 = 17 ≥ 16 needed.**
  No genuine shortfall, so no fabrication was necessary; the 5 authored slots were
  filled with real Prep Ch39 MCQs (5 of the 6 available; Example 39-5, a P-V-cycle
  area question, was left unused).

## The 5 replacements (all real questions, `-stem.png` crop excludes the worked Solution)

PDF page = 0-indexed PDF page; (printed) = the book's printed page number.

| Problem id | Replaced with                                                                   | PDF p. (printed) | Answer key | Index |
| ---------- | ------------------------------------------------------------------------------- | ---------------- | ---------- | ----- |
| ch35-c1-p4 | Prep Ch39 **Example 39-3** — "perfectly insulated system, ΔE and Q if W=+100 J" | 967 (956)        | choice A   | 0     |
| ch35-c2-p4 | Prep Ch39 **Example 39-2** — blood-circulation convection                       | 960 (949)        | choice C   | 2     |
| ch35-c3-p4 | Prep Ch39 **Example 39-4** — raise T with lowest heat, no work → isochoric      | 967 (956)        | choice B   | 1     |
| ch35-c4-p3 | Prep Ch39 **Example 39-6** — free expansion of a gas into vacuum                | 969 (958)        | choice D   | 3     |
| ch35-c4-p4 | Prep Ch39 **Example 39-7** — surgery-recovery entropy                           | 970 (959)        | choice A   | 0     |

Each crop shows the verbatim question stem + the four printed A-D choices, cropped
from the Prep PDF and bounded to end just before "Solution:" so the answer is not
revealed. `choices` are transcribed to text for the answer buttons; `correctIndex`
is the printed answer-key letter. Problem ids were reused, so the deck shape is
unchanged.

## Final shape validation (post-fix)

5 leaves × 4 concepts × 4 problems = **80**; **0** problems without an image crop;
every problem has exactly 4 choices with `correctIndex` in 0-3; **108** unique ids;
all **108** referenced images (20 concept `-src.png` + 80 problem `-stem.png` + 8
choice `-c{i}.png`) present on disk; only the sibling's `placeholder-a/b.png` are
unreferenced (left untouched).
