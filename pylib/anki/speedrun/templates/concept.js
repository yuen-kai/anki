// SpeedrunConcept card behaviour. Contrasting cases, then tell (decision D6):
// the learner states the shared concept, then compares it with the revealed
// one. Pure DOM + sessionStorage, no network or model calls.
(function () {
  "use strict";

  var STORE_KEY = "speedrun:concept:answer";

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

  // The answer block is appended after {{FrontSide}}, so its presence is how we
  // tell the front render from the back render (the whole card is parsed before
  // scripts re-run, so this check is reliable on both desktop and AnkiDroid).
  var answer = document.querySelector('.sr-concept--answer');
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
