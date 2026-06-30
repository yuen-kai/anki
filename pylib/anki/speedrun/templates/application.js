// SpeedrunApplication card behaviour: the principle-first scaffold (decision
// D5). The learner narrows foundation -> category -> topic; a wrong pick shows
// that node's discriminating cue and re-prompts the same level; a complete
// correct path unlocks the worked solution. Entirely data-driven from the
// note's JSON fields, with no fetch / XHR / model calls. `pycmd` is the desktop
// Qt bridge (not network) and is only touched when present, so mobile and the
// answer side degrade cleanly.
(function () {
  "use strict";

  var STORE_KEY = "speedrun:app:trail";
  var LEVEL_PROMPTS = {
    foundation: "Which foundation does this test?",
    category: "Which content category?",
    topic: "Which topic?"
  };
  var DEFAULT_FB =
    "Not the distinguishing feature here. Look again at what the problem pins down.";

  function parseJSON(id, fallback) {
    var el = document.getElementById(id);
    if (!el) return fallback;
    var text = (el.textContent || "").trim();
    if (!text) return fallback;
    try {
      return JSON.parse(text);
    } catch (e) {
      return fallback;
    }
  }

  function bridge(message) {
    // Desktop-only structured logging / Show-Answer gating seam. No-op elsewhere.
    if (typeof pycmd === "function") {
      try {
        pycmd(message);
      } catch (e) {
        /* ignore */
      }
    }
  }

  var steps = parseJSON("sr-steps", []);
  var correctPath = parseJSON("sr-path", []);
  var fbmap = parseJSON("sr-fbmap", {});

  // id -> label, drawn from every option the data offers.
  var labelById = {};
  steps.forEach(function (step) {
    (step.options || []).forEach(function (opt) {
      if (opt && opt.id != null) labelById[opt.id] = opt.label != null ? opt.label : opt.id;
    });
  });

  function labelOf(id) {
    return labelById[id] != null ? labelById[id] : id;
  }

  function saveTrail(labels) {
    try {
      sessionStorage.setItem(STORE_KEY, JSON.stringify(labels));
    } catch (e) {
      /* storage disabled: the answer side just falls back to the correct path */
    }
  }

  function loadTrail() {
    try {
      var raw = sessionStorage.getItem(STORE_KEY);
      var parsed = raw ? JSON.parse(raw) : [];
      return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
      return [];
    }
  }

  function renderTrail(el, labels) {
    if (!el) return;
    el.innerHTML = "";
    labels.forEach(function (label) {
      var li = document.createElement("li");
      li.className = "sr-trail__step";
      li.textContent = label;
      el.appendChild(li);
    });
  }

  // --- Answer side: show the path taken (or the correct path) + solution ----
  if (document.querySelector(".sr-app--answer")) {
    var labels = loadTrail();
    if (!labels.length && correctPath.length) {
      labels = correctPath.map(labelOf);
    }
    renderTrail(document.getElementById("sr-trail"), labels);
    return;
  }

  // --- Question side: the interactive chooser -------------------------------
  var root = document.querySelector(".sr-app");
  var trailEl = document.getElementById("sr-trail");
  var levelEl = document.getElementById("sr-level");
  var feedbackEl = document.getElementById("sr-feedback");
  var unlockedEl = document.getElementById("sr-unlocked");

  var level = 0;
  var chosen = [];
  saveTrail(chosen);

  function clearFeedback() {
    if (!feedbackEl) return;
    feedbackEl.textContent = "";
    feedbackEl.classList.remove("is-visible");
  }

  function showFeedback(message) {
    if (!feedbackEl) return;
    feedbackEl.textContent = message;
    feedbackEl.classList.add("is-visible");
  }

  function complete() {
    renderTrail(trailEl, chosen);
    if (levelEl) levelEl.innerHTML = "";
    clearFeedback();
    if (unlockedEl) unlockedEl.hidden = false;
    if (root) root.setAttribute("data-scaffold-complete", "true");
    window.speedrunScaffoldComplete = true;
    bridge("speedrun:scaffold:complete");
  }

  function onPick(step, opt, btn) {
    var correct = opt.id === step.correct;
    bridge("speedrun:pick:" + step.level + ":" + (correct ? "1" : "0"));
    if (correct) {
      chosen.push(opt.label);
      saveTrail(chosen);
      clearFeedback();
      level += 1;
      renderLevel();
    } else {
      showFeedback(fbmap[opt.id] || DEFAULT_FB);
      btn.classList.add("is-wrong");
    }
  }

  function renderLevel() {
    if (level >= steps.length) {
      complete();
      return;
    }
    renderTrail(trailEl, chosen);
    var step = steps[level];
    if (!levelEl) return;
    levelEl.innerHTML = "";

    var heading = document.createElement("p");
    heading.className = "sr-level__q";
    heading.textContent = LEVEL_PROMPTS[step.level] || "Choose:";
    levelEl.appendChild(heading);

    var list = document.createElement("div");
    list.className = "sr-options";
    (step.options || []).forEach(function (opt) {
      var btn = document.createElement("button");
      btn.type = "button";
      btn.className = "sr-option";
      btn.textContent = opt.label;
      btn.setAttribute("data-node", opt.id);
      btn.addEventListener("click", function () {
        onPick(step, opt, btn);
      });
      list.appendChild(btn);
    });
    levelEl.appendChild(list);

    var first = list.querySelector("button");
    if (first) {
      try {
        first.focus({ preventScroll: true });
      } catch (e) {
        first.focus();
      }
    }
  }

  if (!steps.length) {
    // No authored scaffold: leave a plain card rather than an empty widget.
    if (root) root.classList.add("sr-app--unscaffolded");
    return;
  }

  renderLevel();
})();
