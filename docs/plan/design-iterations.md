# Speedrun: design iterations

> Dated, feedback-driven design changes over time. Append-only. Each entry has a status + category and cross-references the decision it changed. (Pre-build planning choices live in [decisions.md](decisions.md); this log captures changes prompted by feedback and, later, by testing.)

---

### 2026-06-30: Concept prompt asks for the shared concept, not the difference

- **Category:** pedagogy · **Status:** applied · **Prompted by:** user feedback during planning.
- **Change:** the contrasting-cases card asks "what's the shared underlying concept?" rather than "what differs?". Better matches Gick & Holyoak (schema induced by articulating analog *similarity*).
- **Ref:** [D6](decisions.md#d6); [`spec-study-model.md`](spec-study-model.md) §3, §6.

### 2026-06-30: Scaffold is a hierarchy, not a single pick

- **Category:** pedagogy · **Status:** applied · **Prompted by:** user feedback during planning.
- **Change:** the principle-first step drills Foundation → Content Category → Topic (the Dufresne/Mestre HAT) instead of one flat principle choice.
- **Ref:** [D5](decisions.md#d5); [`spec-study-model.md`](spec-study-model.md) §5.

### 2026-06-30: Mode labels reframed to Learn / Practice

- **Category:** UX copy · **Status:** superseded (see below) · **Prompted by:** user feedback during planning.
- **Change:** user-facing modes renamed from Focused/Review to **Learn** / **Practice** (plain user verbs; "Review" collided with Anki's generic term). Mechanical terms (blocked/interleaved) stay in the specs.
- **Ref:** [D20](decisions.md#d20); applied across PRD + specs.

### 2026-06-30: One Study button; the progression picks the mode

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("the learn and review buttons should be merged").
- **Change:** the two-button Learn/Practice split is gone. One **Study** button arms the state-aware topic-grouped queue, which serves blocked→mixed for Speedrun topics and falls back to the normal queue otherwise, so the learner never picks a mode. `learn`/`practice` remain as link aliases for older callers.
- **Ref:** [D30](decisions.md#d30) supersedes [D20](decisions.md#d20); `qt/aqt/overview.py`, `ftl` `studying-study`.

### 2026-06-30: Topics view = a four-stage mastery ladder

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("a UI listing all topics and which of the 4 stages each is at").
- **Change:** the dashboard gained a Topics section. Its signature is a per-topic four-segment track filled left to right up to the topic's current rung (learning → practicing → applying → mastering), grouped under the real AAMC hierarchy, with a ladder summary counting topics on each rung. The internal `hierarchy` state is shown to the learner as **Applying** (the mechanism name isn't a stage a student recognises). One accent, spent on the track and the occupied rungs.
- **Ref:** `ts/routes/speedrun-dashboard/TopicsProgress.svelte`, `lib.ts` `buildTopicsView`; `proto` `TopicProgress.path`.

### 2026-06-30: Topic breadcrumb on the card

- **Category:** UX · **Status:** applied · **Prompted by:** user feedback ("show the topic/hierarchy when learning").
- **Change:** each Speedrun card shows its AAMC path (foundation → category → topic) as a breadcrumb. Labels come from the taxonomy itself (`topic_path_labels`), delivered by the new `GetSpeedrunCardContext` RPC and injected as `window.speedrunTopicPath` (rendered with `textContent`, so a label can never inject markup).
- **Ref:** `rslib/src/speedrun/taxonomy.rs`, `proto` `GetSpeedrunCardContext`, `qt/aqt/reviewer.py`, card templates.

---

<sub>Maintained with the `iris-log` skill by Iris Cai.</sub>
