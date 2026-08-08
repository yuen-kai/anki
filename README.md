# Police Chase

A standalone browser game. No build step, no runtime dependencies: plain ES
modules, with three.js vendored under `vendor/`.

## Running it

```bash
npm start
```

Then open <http://localhost:8123/>. Any static file server works — the only
requirement is that the files are served over HTTP rather than opened as
`file://`, because the page uses ES modules and an import map.

Add `?quality=low`, `?quality=medium` or `?quality=high` to the URL to override
the automatically detected quality tier.

## Where the design lives

`DESIGN.md` is the single source of truth for every gameplay design decision,
and it contains only the verbatim text of the person specifying the game. No
gameplay design is recorded anywhere else — not in this README, not in code
comments, not in commit messages.

`QUESTIONS.md` is the working questionnaire used to collect those decisions. It
is not authoritative; it exists so that short answers stay interpretable and so
that open questions are visible. Anything still unanswered there is either
unbuilt or built as an explicitly marked placeholder.

## Layout

```
index.html            page shell and import map
serve.mjs             zero-dependency static server
src/
  main.js             bootstrap and frame loop wiring
  pendingAnswers.js   values still waiting on a design answer
  engine/             renderer setup, fixed-timestep loop, keyboard, math
  game/               vehicles, police unit, arrest, HUD, run flow
  render/             sky and daylight, chase camera, car models
  world/              map data, road network, nav graph, ground, buildings
vendor/three/         three.js (MIT), vendored so the game runs offline
```

`src/game/tuning.js` and `src/game/vehicleProfiles.js` hold the numbers behind
the chase, kept in one place so they can be changed without touching logic.
`src/pendingAnswers.js` holds anything not yet specified at all; those values
are printed on screen with the question they are waiting on.

`src/world/mapData.js` holds the authored map: road centrelines, park and lot
footprints, and the map extents. `src/world/roadNetwork.js` samples those
centrelines, finds junctions, and validates on load that no road endpoint is
left unconnected; problems are reported to the console.

## Quality tiers

`src/engine/renderer.js` picks a tier from device hints, scaling shadow map
size, the ground mask resolution, texture size and anisotropy. The tier is
exposed at `window.policeChase.quality` along with the scene, renderer and
world data for debugging.
