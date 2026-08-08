# Open design questions

This file is **not** authoritative. `DESIGN.md` is the sole source of gameplay
design truth and contains only the user's verbatim text. This file exists only
to hold the questions being asked, and to keep the numbering stable so short
answers stay interpretable.

Answer format: reply with the question number and your answer, e.g. `1.2 b`.
Lettered options are menus, not recommendations — none of them is a default, and
"something else" is always available. Anything left unanswered stays unbuilt.

Status legend: `[ ]` unanswered, `[x]` answered and copied into `DESIGN.md`.

---

## 1. View and presentation

- [x] 1.1 Camera view: (a) top-down 2D looking straight down, (b) angled/isometric
      2.5D, (c) 3D chase camera behind the car, (d) something else.
- [x] 1.2 Camera rotation: (a) world stays fixed and the car sprite rotates,
      (b) camera rotates so the car always points up the screen, (c) something else.
- [x] 1.3 Camera framing: (a) car locked to screen centre, (b) camera leads ahead
      in the direction of travel, (c) camera zooms out as speed rises,
      (d) some combination — say which.
- [x] 1.4 Art style: (a) flat geometric shapes/vector, (b) pixel art, (c) drawn
      sprites with detail, (d) something else.
- [x] 1.5 How much of the city should be visible at once — roughly how many city
      blocks across the screen? (delegated to the implementer)
- [x] 1.6 Time of day: (a) daytime, (b) night with headlights and light pools,
      (c) it changes during a run, (d) something else.
- [x] 1.7 Should police cars have visibly flashing siren lights, and should those
      lights cast light onto the road?

## 2. The city

- [x] 2.1 World type: (a) endless procedurally generated city, (b) one fixed
      hand-made map, (c) fixed map that wraps around at the edges, (d) something else.
- [x] 2.2 If the map is finite, what stops the player at the edge — walls, water,
      map boundary that turns you back, or something else?
- [x] 2.3 Street layout: (a) strict regular grid, (b) grid with irregularities
      (varied block sizes, occasional diagonals, dead ends), (c) something else.
- [x] 2.4 Should there be alleys/shortcuts that the player can fit through but
      police cannot, or that only some police can?
- [x] 2.5 Are buildings solid walls, or can the player cut across some terrain
      (parking lots, parks, plazas) at a speed penalty?
- [x] 2.5b Follow-up: parks/lots exist and buildings are solid — when you drive
      across a park or lot, does it slow you down, handle the same as road, or
      something else? Same answer for police?
- [ ] 2.5c Are street trees and street lamps solid obstacles, or scenery you
      drive through? Same answer for police?
- [x] 2.6 Are there one-way streets, traffic lights, or other road rules that
      matter mechanically?
- [x] 2.7 Civilian traffic: (a) none, (b) present as moving obstacles, (c) present
      and can be used to block police. If present, what happens when the player
      hits one?
- [x] 2.8 Pedestrians: present or not, and if present what happens on contact?

## 3. Player car — "more control"

- [x] 3.1 What does "more control" mean concretely? Pick all that apply:
      (a) tighter turning radius, (b) quicker acceleration, (c) stronger braking,
      (d) handbrake/drift ability police don't have, (e) better grip so it doesn't
      slide, (f) can reverse quickly, (g) something else.
- [x] 3.2 Handling model: (a) arcade — car turns at a fixed rate and always goes
      where it points, (b) momentum/grip model where you can slide and oversteer,
      (c) something else.
- [ ] 3.3 Is there a handbrake for sharp turns? If yes, does it cost speed?
      (3.1 "all" includes a handbrake the police lack, so this is now only about
      whether using it costs speed, and which key it uses.)
- [ ] 3.4 Can the player reverse? Does the camera/steering invert when reversing?
      (3.1 "all" includes fast reverse, so this is now only about the camera.)
- [ ] 3.5 Is throttle binary (pressed/not) or does the car coast and decelerate
      when you let go?
- [ ] 3.6 Is there a boost/nitro? If yes: what refills it (time, drifting, near
      misses, pickups), how long does it last, and how much faster is it?
- [x] 3.7 Top speed comparison: police should be faster — by roughly how much?
      Give a number if you have one (e.g. "police are 15% faster on the straight").
      (delegated to the implementer)
- [ ] 3.8 Does the player car take damage that affects handling, or is it always
      in perfect condition until the run ends?

## 4. Police cars

- [ ] 4.1 Police handling: (a) same model as the player but with worse turning,
      (b) they understeer and overshoot corners, (c) they're only limited by top
      speed, not cornering, (d) something else.
- [ ] 4.2 Are all police cars identical, or are there types (fast interceptor,
      heavy rammer, blocker, motorcycle, helicopter, etc.)? If types, when do
      they appear and what does each do?
- [ ] 4.3 Can police cars crash — into buildings, traffic, or each other? If yes,
      are they destroyed, stunned for a while, or do they just bounce off?
- [ ] 4.4 If a police car is destroyed or disabled, does it count against the
      current police count, and does it come back?
- [ ] 4.5 Can the player deliberately destroy police cars (ramming, leading them
      into obstacles)? Should that be rewarded, punished, or neutral?
- [ ] 4.6 Do police collide with each other, or pass through each other?

## 5. The arrest rule (the lose condition)

- [x] 5.1 What exactly triggers an arrest? (a) any police car touches you,
      (b) a police car stays within a radius for N seconds, (c) you are surrounded
      by N police cars, (d) you are stopped/slow with police adjacent,
      (e) an "arrest meter" fills up, (f) something else.
- [ ] 5.2 If it's proximity-based: how close, and for how long? Both numbers are
      still needed: how small is the radius in metres (a car is about 4.3m long),
      and what is N in seconds?
- [ ] 5.3 If it's a meter: how fast does it fill, does more police nearby fill it
      faster, does it drain when you get away, and how fast does it drain?
      (5.1 says the timer resets when police are gone, so this is now only about
      whether several police inside the radius make it run faster than one.)
- [x] 5.4 Is arrest instant game over, or do you get chances/lives/a struggle
      mini-window to break free?
- [x] 5.5 Should there be a visible warning before arrest happens (screen effect,
      sound, meter), and how much warning? (answered by 10.1)
- [ ] 5.6 Can the player be arrested at high speed, or does being fast protect you?
- [x] 5.7 Does crashing into a building end the run, slow you down, damage you, or
      nothing?
- [ ] 5.7b Buildings are solid and hitting one does nothing. Does the car scrape
      along the wall keeping the speed it had, or come to a stop against it?

## 6. Police cooperation and trapping

- [x] 6.1 Which trapping tactics should the police use? Pick all that apply:
      (a) direct pursuit from behind, (b) intercept — drive to where you'll be,
      not where you are, (c) pincer from two sides of a block, (d) cut off at
      intersections ahead, (e) roadblocks that appear on streets ahead,
      (f) spike strips, (g) PIT manoeuvre / ramming to spin you out,
      (h) herding — deliberately leaving one route open to push you somewhere,
      (i) full encirclement, (j) something else.
- [ ] 6.1b Spike strips are in. What does driving over one do to the player —
      slow you for a while, stop you, or something else? Does it affect police
      who drive over it? How is one deployed, and is it visible in advance?
- [ ] 6.1c Roadblocks are in. Are they police cars parked across a street ahead,
      or separate barricade props? What happens when the player hits one?
- [x] 6.2 Should the police act as one coordinated unit with assigned roles
      (e.g. two chasers + two flankers), or should each car decide for itself
      with some awareness of the others?
- [ ] 6.2b Which roles should the unit assign between? Pursue directly, intercept
      ahead, flank on a parallel street, block an intersection, deploy a spike
      strip, form a roadblock, hold back and contain — or a different set?
- [ ] 6.3 Should coordination get smarter as the run goes on, or be equally smart
      from the start with only the count increasing?
- [ ] 6.4 Do police always know exactly where the player is, or do they need line
      of sight / a last-known position and have to search?
- [ ] 6.5 If they can lose you: how do you break line of sight, how long until
      they give up, and what happens then — do they patrol, converge on your last
      position, or leave?
- [ ] 6.6 Should the player ever be able to fully escape and reset the chase, or
      is the pressure permanent until arrest?
- [ ] 6.7 Should the player be able to see police intentions (e.g. a roadblock
      warning, or a marker on a car that is about to cut you off)?

## 7. Spawning and difficulty ramp

- [x] 7.1 How many police cars at the very start of a run?
- [x] 7.2 How do more arrive — on a timer, at score thresholds, in waves,
      or something else? Give the schedule if you have one.
- [ ] 7.2b What is the timer interval — one new car every how many seconds, and
      is the interval constant or does it shorten as the run goes on?
- [ ] 7.3 Is there a maximum number of police cars?
- [x] 7.4 Where do new police enter from: (a) off-screen edges, (b) out of side
      streets near you, (c) from fixed police stations on the map,
      (d) something else.
- [ ] 7.5 Should there be a grace period at the start before any police engage?
- [ ] 7.6 Besides adding cars, should difficulty ramp in other ways (police get
      faster, smarter, new types unlock, tactics change)?
- [ ] 7.7 Is there a "wanted level" concept that steps up visibly, or is the ramp
      continuous and unlabelled?

## 8. Score and run structure

- [x] 8.1 Is score purely survival time, or are there other sources (distance,
      near misses, drifting, evading, destroying police cars)? If more than time,
      how much is each worth relative to a second of survival?
- [ ] 8.2 Is there any way to win, or is every run intended to end in arrest?
- [ ] 8.3 Should high scores be saved locally in the browser, and should the
      run-end screen show a personal best?
- [ ] 8.4 What stats should the end-of-run screen show?
- [ ] 8.5 Should runs be seeded/repeatable, or fully random each time?

## 9. Pickups and extras

- [ ] 9.1 Any pickups on the map at all? If yes, which: repair, boost, hideout
      that clears the chase, EMP, spike-strip-of-your-own, something else?
- [ ] 9.2 If there are hideouts/garages, how do they work and how often can they
      be used?
- [ ] 9.3 Anything else the player can do besides drive (horn, handbrake turn,
      dropping obstacles, shooting)?

## 10. HUD and screens

- [x] 10.1 What should the HUD show? Pick all that apply: (a) survival timer,
      (b) score, (c) speedometer, (d) minimap, (e) number of police in pursuit,
      (f) arrest/capture meter, (g) wanted level, (h) damage, (i) something else.
- [ ] 10.2 If there's a minimap: does it show all police, only nearby ones, or
      only ones that can see you? Does it show the street layout?
- [ ] 10.3 Should there be off-screen indicators pointing at nearby police?
- [ ] 10.4 Start screen: straight into the game, or a title screen with a start
      button and controls listed?
- [ ] 10.5 Should there be a countdown or intro before the chase begins?
- [ ] 10.6 Can the game be paused? If yes, does pausing feel like cheating —
      should it be blocked during a chase?
- [ ] 10.7 What should happen on arrest — instant cut to the score screen, a slow
      motion moment, a short cinematic of the surround, something else?
- [ ] 10.8 Restart: single key press to go again, or back to the menu?

## 11. Audio

- [ ] 11.1 Should there be sound at all in the first version?
- [ ] 11.2 If yes, which: engine, sirens (louder when close), tyre screech,
      crashes, music, arrest sting?
- [ ] 11.3 Should siren volume/panning be a gameplay signal for where police are?

## 12. Controls and platform

- [x] 12.1 Keyboard layout: arrows, WASD, both, or something else? Which key for
      handbrake/boost if those exist?
- [ ] 12.1b WASD is the driving input. Which key is the handbrake?
- [ ] 12.2 Should the game support touch (mobile) and/or gamepad?
- [ ] 12.3 Is mouse steering wanted as an option?
- [ ] 12.4 Any accessibility requirements (colour-blind safe palette, reduced
      screen shake, adjustable difficulty)?

---

## Answering order

Sections 1 and 2 are answered, as is the first pass of blocking questions.
The follow-ups that came out of those answers and still block a complete build
are: 5.2, 6.1b, 6.1c, 6.2b, 7.2b, 10.7, 10.8. The rest can be layered on.
