// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

// DOM-stub tests for the state-aware Speedrun card templates (decision D31).
//
// These drive the REAL concept.js / application.js against a tiny hand-rolled
// DOM (no build, no jsdom: just `node`) once per card mode, to prove each branch
// the reviewer selects via window.speedrunCardMode:
//   concept_learn / concept_practice / application_scaffolded /
//   application_unscaffolded, plus the safe fallback for an absent or unknown
//   mode. It also replicates the reviewer's Show-Answer gate probe
//   (qt/aqt/speedrun.py GATE_PROBE_JS) to show the unscaffolded card fails the
//   gate open, and audits every template asset for network / model calls.
//
// Run:  node pylib/tests/test_speedrun_card_modes.mjs

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const TEMPLATES = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "anki",
  "speedrun",
  "templates",
);

const CONCEPT_JS = readFileSync(join(TEMPLATES, "concept.js"), "utf8");
const APPLICATION_JS = readFileSync(join(TEMPLATES, "application.js"), "utf8");

// --- minimal DOM ----------------------------------------------------------

class El {
  constructor(tag) {
    this.tagName = (tag || "").toUpperCase();
    this.children = [];
    this.parent = null;
    this._classes = new Set();
    this.attrs = {};
    this._text = "";
    this.hidden = false;
    this.listeners = {};
    this.id = "";
    this.value = "";
    this.readOnly = false;
    this.type = "";
  }
  get className() {
    return Array.from(this._classes).join(" ");
  }
  set className(v) {
    this._classes = new Set(String(v).split(/\s+/).filter(Boolean));
  }
  get classList() {
    const self = this;
    return {
      add() {
        for (const c of arguments) self._classes.add(c);
      },
      remove() {
        for (const c of arguments) self._classes.delete(c);
      },
      contains(c) {
        return self._classes.has(c);
      },
    };
  }
  setAttribute(k, v) {
    this.attrs[k] = String(v);
    if (k === "id") this.id = String(v);
  }
  getAttribute(k) {
    return k in this.attrs ? this.attrs[k] : null;
  }
  appendChild(c) {
    c.parent = this;
    this.children.push(c);
    return c;
  }
  get textContent() {
    if (this.children.length) {
      return this.children.map((c) => c.textContent).join("");
    }
    return this._text;
  }
  set textContent(v) {
    this._text = String(v);
    this.children = [];
  }
  get innerHTML() {
    return "";
  }
  set innerHTML(v) {
    if (v === "") this.children = [];
    this._text = "";
  }
  addEventListener(type, fn) {
    (this.listeners[type] = this.listeners[type] || []).push(fn);
  }
  focus() {
    /* noop */
  }
  querySelector(sel) {
    return query(this, sel, true);
  }
  querySelectorAll(sel) {
    return query(this, sel, false);
  }
  // test helper: fire registered listeners of a type
  fire(type) {
    (this.listeners[type] || []).forEach((fn) => fn({}));
  }
}

function matchSimple(el, s) {
  if (!el || !el.tagName) return false;
  if (s[0] === "#") return el.id === s.slice(1);
  if (s[0] === ".") return el._classes.has(s.slice(1));
  return el.tagName === s.toUpperCase();
}

function descendants(root) {
  const out = [];
  (function walk(n) {
    (n.children || []).forEach((c) => {
      out.push(c);
      walk(c);
    });
  })(root);
  return out;
}

function matchChain(el, parts) {
  if (!matchSimple(el, parts[parts.length - 1])) return false;
  let i = parts.length - 2;
  let anc = el.parent;
  while (i >= 0 && anc) {
    if (matchSimple(anc, parts[i])) i--;
    anc = anc.parent;
  }
  return i < 0;
}

function query(root, sel, single) {
  const parts = sel.trim().split(/\s+/);
  const res = descendants(root).filter((el) => matchChain(el, parts));
  return single ? res[0] || null : res;
}

// el("div", { class, id, text, attrs, value, type }, [children])
function el(tag, opts = {}, children = []) {
  const node = new El(tag);
  if (opts.class) node.className = opts.class;
  if (opts.id) node.setAttribute("id", opts.id);
  if (opts.attrs) for (const [k, v] of Object.entries(opts.attrs)) node.setAttribute(k, v);
  if (opts.type) node.type = opts.type;
  if (opts.value != null) node.value = opts.value;
  if (opts.hidden) node.hidden = true;
  if (opts.text != null) node.textContent = opts.text;
  children.forEach((c) => node.appendChild(c));
  return node;
}

function makeDocument(roots) {
  const root = new El("#document");
  roots.forEach((r) => root.appendChild(r));
  const byId = {};
  descendants(root).forEach((n) => {
    if (n.id) byId[n.id] = n;
  });
  return {
    root,
    getElementById: (id) => byId[id] || null,
    querySelector: (sel) => query(root, sel, true),
    querySelectorAll: (sel) => query(root, sel, false),
    createElement: (tag) => new El(tag),
  };
}

function makeSessionStorage() {
  const store = {};
  return {
    getItem: (k) => (k in store ? store[k] : null),
    setItem: (k, v) => {
      store[k] = String(v);
    },
    removeItem: (k) => {
      delete store[k];
    },
  };
}

// Run the real template IIFE against supplied globals. `new Function` gives the
// source plain global scope, so its bare `document` / `window` / `sessionStorage`
// resolve to the ones we install (and `pycmd` stays undefined: no bridge).
function runTemplate(source, { document, window, sessionStorage }) {
  globalThis.document = document;
  globalThis.window = window;
  globalThis.sessionStorage = sessionStorage;
  delete globalThis.pycmd;
  // eslint-disable-next-line no-new-func
  new Function(source)();
}

// --- template DOM builders (mirror the .html templates) -------------------

function conceptFront() {
  return el("div", { class: "speedrun sr-concept", attrs: { "data-side": "front" } }, [
    el("div", { class: "sr-cases" }, [
      el("section", { class: "sr-case sr-case--a" }, [
        el("h2", { class: "sr-case__label", text: "Case A" }),
        el("div", { class: "sr-case__body", text: "case a body" }),
      ]),
      el("section", { class: "sr-case sr-case--b" }, [
        el("h2", { class: "sr-case__label", text: "Case B" }),
        el("div", { class: "sr-case__body", text: "case b body" }),
      ]),
    ]),
    el("p", { class: "sr-prompt", text: "what do both share?" }),
    el("label", { class: "sr-answer__label", attrs: { for: "sr-similarity" }, text: "What's the shared underlying concept?" }),
    el("textarea", { class: "sr-answer__input", id: "sr-similarity", type: "textarea" }),
  ]);
}

function conceptAnswerBlock() {
  return el("div", { class: "speedrun sr-concept sr-concept--answer", attrs: { "data-side": "back" } }, [
    el("div", { class: "sr-reveal" }, [
      el("h2", { class: "sr-reveal__label", text: "Your answer" }),
      el("div", { class: "sr-your-answer", id: "sr-your-answer" }, [el("span", { class: "sr-blank", text: "left blank" })]),
    ]),
    el("div", { class: "sr-reveal" }, [
      el("h2", { class: "sr-reveal__label", text: "Shared concept" }),
      el("div", { class: "sr-concept__body", text: "the concept" }),
    ]),
  ]);
}

const SAMPLE_STEPS = [
  { level: "foundation", options: [{ id: "F", label: "Foundation F" }, { id: "F2", label: "Foundation F2" }], correct: "F" },
  { level: "category", options: [{ id: "C", label: "Category C" }, { id: "C2", label: "Category C2" }], correct: "C" },
  { level: "topic", options: [{ id: "T", label: "Topic T" }, { id: "T2", label: "Topic T2" }], correct: "T" },
];
const SAMPLE_PATH = ["F", "C", "T"];
const SAMPLE_FB = { F2: "wrong foundation", C2: "wrong category", T2: "wrong topic" };

function appJsonScripts() {
  return [
    el("script", { id: "sr-steps", attrs: { type: "application/json" }, text: JSON.stringify(SAMPLE_STEPS) }),
    el("script", { id: "sr-path", attrs: { type: "application/json" }, text: JSON.stringify(SAMPLE_PATH) }),
    el("script", { id: "sr-fbmap", attrs: { type: "application/json" }, text: JSON.stringify(SAMPLE_FB) }),
  ];
}

function appFront() {
  return el("div", { class: "speedrun sr-app", attrs: { "data-side": "front" } }, [
    el("div", { class: "sr-problem", text: "the problem" }),
    el("div", { class: "sr-scaffold" }, [
      el("ol", { class: "sr-trail", id: "sr-trail" }),
      el("div", { class: "sr-level", id: "sr-level" }),
      el("p", { class: "sr-feedback", id: "sr-feedback" }),
      el("p", { class: "sr-unlocked", id: "sr-unlocked", hidden: true, text: "Path complete. Reveal the worked solution." }),
    ]),
    ...appJsonScripts(),
  ]);
}

function appBack() {
  return el("div", { class: "speedrun sr-app sr-app--answer", attrs: { "data-side": "back" } }, [
    el("div", { class: "sr-problem", text: "the problem" }),
    el("ol", { class: "sr-trail sr-trail--final", id: "sr-trail" }),
    el("hr", { class: "sr-rule", id: "answer" }),
    el("h2", { class: "sr-reveal__label", text: "Worked solution" }),
    el("div", { class: "sr-solution", text: "the solution" }),
    el("script", { id: "sr-steps", attrs: { type: "application/json" }, text: JSON.stringify(SAMPLE_STEPS) }),
    el("script", { id: "sr-path", attrs: { type: "application/json" }, text: JSON.stringify(SAMPLE_PATH) }),
  ]);
}

// Replica of qt/aqt/speedrun.py GATE_PROBE_JS, run against our stub.
function gateProbe(window, document) {
  if (window.speedrunScaffoldComplete === true) return "complete";
  const node = document.querySelector(".sr-app");
  if (!node) return "absent";
  if (node.getAttribute("data-scaffold-complete") === "true") return "complete";
  if (node.classList.contains("sr-app--unscaffolded")) return "unscaffolded";
  return "incomplete";
}

// --- tiny assert harness --------------------------------------------------

let passed = 0;
const failures = [];
function check(name, cond, detail) {
  if (cond) {
    passed++;
  } else {
    failures.push(name + (detail ? ` (${detail})` : ""));
  }
}

// --- concept tests --------------------------------------------------------

function testConceptLearn(modeValue, label) {
  const front = conceptFront();
  const document = makeDocument([front]);
  const window = { speedrunCardMode: modeValue };
  runTemplate(CONCEPT_JS, { document, window, sessionStorage: makeSessionStorage() });

  check(`${label}: both cases visible`, document.querySelector(".sr-case--b").hidden === false);
  check(`${label}: comparison prompt visible`, document.querySelector(".sr-prompt").hidden === false);
  check(`${label}: case A label unchanged`, document.querySelector(".sr-case--a .sr-case__label").textContent === "Case A");
  check(`${label}: not flagged practice`, document.querySelector(".sr-concept").classList.contains("sr-concept--practice") === false);
}

function testConceptPractice() {
  const session = makeSessionStorage();
  const window = { speedrunCardMode: "concept_practice" };

  // front
  const front = conceptFront();
  const docFront = makeDocument([front]);
  runTemplate(CONCEPT_JS, { document: docFront, window, sessionStorage: session });

  check("concept_practice: root flagged", docFront.querySelector(".sr-concept").classList.contains("sr-concept--practice"));
  check("concept_practice: case B hidden", docFront.querySelector(".sr-case--b").hidden === true);
  check("concept_practice: comparison prompt hidden", docFront.querySelector(".sr-prompt").hidden === true);
  check("concept_practice: case relabelled", docFront.querySelector(".sr-case--a .sr-case__label").textContent === "Example");
  check("concept_practice: recall label", docFront.querySelector(".sr-answer__label").textContent === "Recall the concept");
  check("concept_practice: case A still shown", docFront.querySelector(".sr-case--a").hidden === false);

  // learner types an answer, then reveals
  const box = docFront.getElementById("sr-similarity");
  box.value = "buffering peaks near pKa";
  box.fire("input");

  const back = makeDocument([conceptFront(), conceptAnswerBlock()]);
  runTemplate(CONCEPT_JS, { document: back, window, sessionStorage: session });

  check("concept_practice: answer echoed on back", back.getElementById("sr-your-answer").textContent === "buffering peaks near pKa");
  check("concept_practice: case B hidden on back too", back.querySelector(".sr-case--b").hidden === true);
  check("concept_practice: concept revealed", back.querySelector(".sr-concept__body").textContent === "the concept");
}

// --- application tests ----------------------------------------------------

function pickCorrectThrough(document) {
  // Click the correct option at each level until the scaffold completes.
  for (let i = 0; i < SAMPLE_STEPS.length; i++) {
    const want = SAMPLE_STEPS[i].correct;
    const buttons = document.getElementById("sr-level").querySelectorAll("button");
    const btn = buttons.find((b) => b.getAttribute("data-node") === want);
    if (!btn) return false;
    btn.fire("click");
  }
  return true;
}

function testApplicationScaffolded(modeValue, label) {
  const session = makeSessionStorage();
  const window = { speedrunCardMode: modeValue };

  const front = appFront();
  const docFront = makeDocument([front]);
  runTemplate(APPLICATION_JS, { document: docFront, window, sessionStorage: session });

  check(`${label}: scaffold visible`, docFront.querySelector(".sr-scaffold").hidden === false);
  check(`${label}: not flagged unscaffolded`, docFront.querySelector(".sr-app").classList.contains("sr-app--unscaffolded") === false);
  check(`${label}: chooser rendered`, docFront.getElementById("sr-level").querySelectorAll("button").length === 2);
  check(`${label}: gate blocks before completion`, gateProbe(window, docFront) === "incomplete");

  // a wrong pick shows the authored cue and does not advance
  const wrong = docFront.getElementById("sr-level").querySelectorAll("button").find((b) => b.getAttribute("data-node") === "F2");
  wrong.fire("click");
  check(`${label}: wrong pick shows cue`, docFront.getElementById("sr-feedback").textContent === "wrong foundation");
  check(`${label}: wrong pick marked`, wrong.classList.contains("is-wrong"));
  check(`${label}: still on first level`, docFront.getElementById("sr-level").querySelectorAll("button").find((b) => b.getAttribute("data-node") === "F") != null);

  // complete the correct path
  check(`${label}: completed path`, pickCorrectThrough(docFront));
  check(`${label}: completion flag set`, window.speedrunScaffoldComplete === true);
  check(`${label}: completion attr set`, docFront.querySelector(".sr-app").getAttribute("data-scaffold-complete") === "true");
  check(`${label}: gate opens after completion`, gateProbe(window, docFront) === "complete");
  check(`${label}: unlock note shown`, docFront.getElementById("sr-unlocked").hidden === false);

  // answer side renders the path taken
  const back = makeDocument([appBack()]);
  runTemplate(APPLICATION_JS, { document: back, window, sessionStorage: session });
  const steps = back.getElementById("sr-trail").querySelectorAll(".sr-trail__step");
  check(`${label}: trail rendered on back`, steps.length === 3);
  check(`${label}: trail labels correct`, steps.map((s) => s.textContent).join(",") === "Foundation F,Category C,Topic T");
  check(`${label}: trail visible on back`, back.getElementById("sr-trail").hidden === false);
}

function testApplicationUnscaffolded() {
  const session = makeSessionStorage();
  const window = { speedrunCardMode: "application_unscaffolded" };

  const front = appFront();
  const docFront = makeDocument([front]);
  runTemplate(APPLICATION_JS, { document: docFront, window, sessionStorage: session });

  check("application_unscaffolded: scaffold hidden", docFront.querySelector(".sr-scaffold").hidden === true);
  check("application_unscaffolded: root flagged", docFront.querySelector(".sr-app").classList.contains("sr-app--unscaffolded") === true);
  check("application_unscaffolded: no chooser rendered", docFront.getElementById("sr-level").querySelectorAll("button").length === 0);
  check("application_unscaffolded: completion flag not set", window.speedrunScaffoldComplete === undefined);
  check("application_unscaffolded: problem still shown", docFront.querySelector(".sr-problem").hidden === false);
  // the load-bearing one: the reviewer gate must fail open
  check("application_unscaffolded: gate fails open", gateProbe(window, docFront) === "unscaffolded");

  const back = makeDocument([appBack()]);
  runTemplate(APPLICATION_JS, { document: back, window, sessionStorage: session });
  check("application_unscaffolded: trail hidden on back", back.getElementById("sr-trail").hidden === true);
  check("application_unscaffolded: solution shown", back.querySelector(".sr-solution").textContent === "the solution");
}

// --- no-network / no-model audit -----------------------------------------

const JS_FORBIDDEN = [
  ["fetch(", /\bfetch\s*\(/],
  ["XMLHttpRequest", /XMLHttpRequest/],
  ["WebSocket", /WebSocket/],
  ["EventSource", /EventSource/],
  ["sendBeacon", /sendBeacon/],
  ["dynamic import()", /\bimport\s*\(/],
  ["importScripts", /importScripts/],
  ["navigator", /\bnavigator\b/],
  ["Worker(", /\bWorker\s*\(/],
  ["external url", /\bhttps?:\/\//i],
  ["socket url", /\bwss?:\/\//i],
];
const MARKUP_FORBIDDEN = [
  ["external url", /\bhttps?:\/\//i],
  ["socket url", /\bwss?:\/\//i],
  ["<script src>", /<script[^>]*\bsrc=/i],
  ["<link>", /<link\b/i],
  ["<iframe>", /<iframe\b/i],
  ["<img>", /<img\b/i],
];

function audit() {
  const jsFiles = ["concept.js", "application.js"];
  const markupFiles = [
    "concept_front.html",
    "concept_back.html",
    "application_front.html",
    "application_back.html",
    "concept.css",
    "application.css",
  ];
  for (const f of jsFiles) {
    const src = readFileSync(join(TEMPLATES, f), "utf8");
    for (const [name, re] of JS_FORBIDDEN) {
      check(`audit ${f}: no ${name}`, !re.test(src), "found a forbidden token");
    }
  }
  for (const f of markupFiles) {
    const src = readFileSync(join(TEMPLATES, f), "utf8");
    for (const [name, re] of MARKUP_FORBIDDEN) {
      check(`audit ${f}: no ${name}`, !re.test(src), "found a forbidden token");
    }
  }
}

// --- structural audit: the JS hooks must exist in the real HTML -----------

function structuralAudit() {
  const cf = readFileSync(join(TEMPLATES, "concept_front.html"), "utf8");
  ["sr-concept", "sr-case--a", "sr-case--b", "sr-prompt", "sr-answer__label", 'id="sr-similarity"'].forEach((hook) =>
    check(`concept_front.html has ${hook}`, cf.includes(hook)),
  );
  const af = readFileSync(join(TEMPLATES, "application_front.html"), "utf8");
  ["sr-app", "sr-scaffold", 'id="sr-trail"', 'id="sr-level"'].forEach((hook) =>
    check(`application_front.html has ${hook}`, af.includes(hook)),
  );
  const ab = readFileSync(join(TEMPLATES, "application_back.html"), "utf8");
  ["sr-app--answer", 'id="sr-trail"', "sr-solution"].forEach((hook) =>
    check(`application_back.html has ${hook}`, ab.includes(hook)),
  );
}

// --- run ------------------------------------------------------------------

testConceptLearn(undefined, "concept_learn (absent mode)");
testConceptLearn("concept_learn", "concept_learn (explicit)");
testConceptLearn("totally-unknown", "concept fallback (unknown mode)");
testConceptLearn("application_unscaffolded", "concept fallback (wrong-family mode)");
testConceptPractice();

testApplicationScaffolded(undefined, "application_scaffolded (absent mode)");
testApplicationScaffolded("application_scaffolded", "application_scaffolded (explicit)");
testApplicationScaffolded("totally-unknown", "application fallback (unknown mode)");
testApplicationScaffolded("concept_practice", "application fallback (wrong-family mode)");
testApplicationUnscaffolded();

audit();
structuralAudit();

if (failures.length) {
  console.error(`FAIL: ${failures.length} check(s) failed, ${passed} passed`);
  failures.forEach((f) => console.error("  - " + f));
  process.exit(1);
}
console.log(`OK: ${passed} checks passed across 4 modes + fallback, gate probe, and no-network audit`);
