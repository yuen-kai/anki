// SpeedrunConcept card behaviour. Two render modes (decision D31), chosen by the
// topic's state, which the reviewer injects as window.speedrunCardMode before
// render:
//   concept_learn    - contrasting cases, then tell (decision D6). The default.
//   concept_practice - lighter recall: one case as a cue, recall the concept,
//                      no re-teaching both cases.
// An absent or unrecognised mode falls back to concept_learn, so the current
// behaviour is never broken. Pure DOM + sessionStorage, no network or model calls.
(function () {
  "use strict";

  var STORE_KEY = "speedrun:concept:answer";

  // Only the practice string switches behaviour; anything else (absent, unknown,
  // or a wrong-family value) stays on the safe concept-learn default.
  var practice = window.speedrunCardMode === "concept_practice";

  function read() {
    try {
      return sessionStorage.getItem(STORE_KEY) || "";
    } catch (e) {
      return "";
    }
  }

  function write(value) {
    try {
      sessionStorage.setItem(STORE_KEY, value);
    } catch (e) {
      /* private mode / storage disabled: degrade to no echo */
    }
  }

  function setText(el, text) {
    if (el) el.textContent = text;
  }

  // Strip the pair down to a single cue + recall ask. Runs on both the question
  // and the answer render (the answer pulls the front in via {{FrontSide}}), so
  // the answer side does not re-teach both cases either.
  function applyPractice() {
    var root = document.querySelector(".sr-concept");
    if (root) root.classList.add("sr-concept--practice");
    var caseB = document.querySelector(".sr-case--b");
    if (caseB) caseB.hidden = true;
    var prompt = document.querySelector(".sr-prompt");
    if (prompt) prompt.hidden = true;
    // The remaining case is a worked example, not one half of a contrast.
    setText(document.querySelector(".sr-case--a .sr-case__label"), "Example");
    setText(document.querySelector(".sr-answer__label"), "Recall the concept");
    var box = document.getElementById("sr-similarity");
    if (box) box.setAttribute("placeholder", "Recall the concept, then reveal.");
  }

  if (practice) applyPractice();

  // The answer block is appended after {{FrontSide}}, so its presence is how we
  // tell the front render from the back render (the whole card is parsed before
  // scripts re-run, so this check is reliable on both desktop and AnkiDroid).
  var answer = document.querySelector(".sr-concept--answer");
  var input = document.getElementById("sr-similarity");

  if (answer) {
    if (input) {
      input.readOnly = true;
      input.classList.add("is-locked");
    }
    var out = document.getElementById("sr-your-answer");
    var saved = read();
    if (out && saved.trim()) {
      out.textContent = saved;
      out.classList.remove("sr-blank");
    }
    return;
  }

  if (input) {
    write("");
    input.addEventListener("input", function () {
      write(input.value);
    });
    if (typeof input.focus === "function") {
      try {
        input.focus({ preventScroll: true });
      } catch (e) {
        input.focus();
      }
    }
  }
})();
