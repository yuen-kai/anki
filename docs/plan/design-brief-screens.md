# Speedrun screens and features

A study app for the MCAT. It runs on both desktop and phone.

## Decks (home)

The first screen. Shows your decks, each with its card counts (new, learning,
due) and its deck actions.

- Open a deck to go to its study screen.
- Create a new deck, which opens Create hierarchy.

  ## Create hierarchy

  Where you build a deck's topic tree.

  - A tree of nodes where you name each level.
  - Pulses up the branches to the root, so it looks like leaves add up to its parents.
  - Leaf nodes (aka topics) open their concepts (aka cards).

    ## Concepts List

    The contents of one leaf: the concepts under it.

    - Open a concept to edit it.

    ## Concept editor

    - Concept title.
    - Content description (the explanation).
    - Practice problems, each with four answer choices and the correct one marked.

## Study screen (per deck)

Opened by choosing a deck. It shows:

- Today's progress: how much is left to study today, and a way to start.
- Three separate scores, never blended: Memory, Performance, and Readiness. Each
  shows a likely range, how much of the exam it covers so far, and the reasons
  behind it. When there is not enough data yet, it shows no number and says what
  is missing.
- The deck's concept tree, with each leaf showing its stage (learn, practice,
  scaffold, unscaffold) and parent nodes showing combined progress from their
  children.
- A More details view with per-subject stats explaining how each score is built.

## Study session

The actual studying. The app decides what to serve next; the user does not pick a
mode.

New topic:

- A new-topic intro announces the topic and animates down the tree to its leaf.

Learning a topic:

- The card shows which of the four stages it is, and how many concepts in the
  topic are learned (for example 3/5).
- It shows two problems from the concept with only their correct answers. The
  user types what the two have in common, then the concept details appear.
- No difficulty rating while learning.
- Only when all concepts in the topic are learned, upgrade the cards together. (But usually cards upgrade individually)

Practicing and applying:

- Practice is free response, not multiple choice.
- Application problems walk a scaffold: principle, then concept, then procedure.
  A wrong pick shows a correction

Grading:

- On graded cards, the user rates how hard the card was

Leveling up:

- When a card gets upgraded, there should be an animation screen declaring that the card has been upgraded. This animation should feel good. Indicate what it is being upgraded from and what it is being upgraded to.

The four stages (learn, practice, scaffold, unscaffold) appear on each card and
on the tree leaves.
