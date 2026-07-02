# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""A complete authored demo deck for exercising the Speedrun study flow.

The bespoke study/review screens read authored content from the
``speedrun_authoring`` store (:mod:`anki.speedrun.authoring`), which is normally
built by hand in the Decks -> Hierarchy -> Concepts UI. That leaves no ready
deck to test with, so this module ships one: a small but complete biochemistry
tree that drives every stage of the mastery ladder.

What "complete" means here (matches what the review screen needs):

- **Depth >= 2** (root -> category -> topic leaf), so the Applying-stage
  scaffold has real levels to pick through.
- **Every leaf holds concepts**, and one leaf (pKa & titration) holds two, so
  the topic-gated learning block is exercised (both must be seen before the
  topic flips learning -> practicing).
- **Every concept has content plus >= 2 problems**: the Learn stage shows the
  first two problems as contrasting cases, and the Applying/Mastering stages
  rotate through them as 4-choice MCQs. Every problem has exactly four choices
  with one marked correct.

The content is pure data (no ``anki`` import at build time); :func:`seed` does
the collection work. Ids are stable strings so a reseed is a no-op and the
materialized cards keep their identity.
"""

from __future__ import annotations

from typing import Any

import anki.collection
from anki.speedrun import authoring, materialize

# A distinct name so the demo never collides with the note-based "Speedrun"
# seed (anki.speedrun.notetypes.add_seed_notes).
DEMO_DECK_NAME = "MCAT Biochemistry (demo)"


def _problem(pid: str, prompt: str, choices: list[str], correct: int) -> dict[str, Any]:
    assert len(choices) == 4, "a problem must have exactly four choices"
    assert 0 <= correct < 4, "correctIndex must point at one of the four choices"
    return {"id": pid, "prompt": prompt, "choices": choices, "correctIndex": correct}


def _concept(
    cid: str, title: str, content: str, problems: list[dict[str, Any]]
) -> dict[str, Any]:
    return {"id": cid, "title": title, "content": content, "problems": problems}


def _leaf(nid: str, title: str, concepts: list[dict[str, Any]]) -> dict[str, Any]:
    return {"id": nid, "title": title, "children": [], "concepts": concepts}


def _branch(nid: str, title: str, children: list[dict[str, Any]]) -> dict[str, Any]:
    return {"id": nid, "title": title, "children": children, "concepts": []}


# --- Amino acids ----------------------------------------------------------

_AA_STRUCTURE = _leaf(
    "sr-demo-aa-structure",
    "Structure & classification",
    [
        _concept(
            "sr-demo-c-aa-structure",
            "Side chains set behavior",
            "An amino acid's behavior is set by its side chain (R group), not the "
            "backbone every residue shares. Classifying the R group as nonpolar, "
            "polar uncharged, acidic, or basic predicts where a residue sits and "
            "what it can bond: hydrophobic side chains bury in the nonpolar core, "
            "polar and charged ones face the water.",
            [
                _problem(
                    "sr-demo-p-aa-structure-1",
                    "A residue is buried in the water-excluding core of a folded "
                    "protein. What does that location predict about its side chain?",
                    [
                        "It is a nonpolar hydrocarbon (e.g. leucine, valine)",
                        "It is acidic and negatively charged",
                        "It is basic and positively charged",
                        "It carries a polar hydroxyl that hydrogen-bonds water",
                    ],
                    0,
                ),
                _problem(
                    "sr-demo-p-aa-structure-2",
                    "You must place a residue on a solvent-exposed loop that "
                    "hydrogen-bonds with water. Which side chain fits?",
                    [
                        "Isoleucine",
                        "Phenylalanine",
                        "Serine or glutamine (polar)",
                        "Leucine",
                    ],
                    2,
                ),
            ],
        ),
    ],
)

_AA_PKA = _leaf(
    "sr-demo-aa-pka",
    "pKa & titration",
    [
        _concept(
            "sr-demo-c-aa-pka",
            "Buffering peaks near pKa",
            "A weak acid or base resists pH change most strongly when the ambient "
            "pH is near its pKa, where it is about half protonated and the "
            "Henderson-Hasselbalch ratio [A-]/[HA] is about 1. Distance from pKa "
            "is exactly what makes a group a poor buffer and leaves it nearly "
            "fully protonated or deprotonated.",
            [
                _problem(
                    "sr-demo-p-aa-pka-1",
                    "Titrating aspartic acid with NaOH, the curve flattens three "
                    "times, each flat zone sitting at a pKa (about 2.0, 3.9, 9.9). "
                    "Why does the pH barely move there?",
                    [
                        "The buffer has been exhausted",
                        "Buffering capacity peaks when pH is near pKa",
                        "Water autoionization takes over",
                        "The amino acid has precipitated",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-aa-pka-2",
                    "You need a side chain that shuttles protons at pH 7.4. Given "
                    "aspartate (pKa 3.9), histidine (6.0), and lysine (10.5), which "
                    "is best?",
                    [
                        "Aspartate, because it is acidic",
                        "Lysine, because it is basic",
                        "Histidine, because its pKa is nearest 7.4",
                        "Any of them, since all are ionizable",
                    ],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-demo-c-aa-pi",
            "Net charge and the isoelectric point",
            "Net charge is fixed by where the ambient pH sits relative to a "
            "molecule's pKa values, summarized by the isoelectric point (pI): the "
            "pH at which positive and negative charges balance to zero net charge. "
            "Below pI a species is net positive, above pI net negative, and at pI "
            "it is neutral and least soluble.",
            [
                _problem(
                    "sr-demo-p-aa-pi-1",
                    "At pH 7.6, lysine (pI 9.7), aspartate (pI 2.8), and histidine "
                    "(pI 7.6) are placed in an electric field. Which barely moves?",
                    [
                        "Lysine, it runs to the cathode",
                        "Aspartate, it runs to the anode",
                        "Histidine, the running pH sits at its pI",
                        "They all migrate at the same speed",
                    ],
                    2,
                ),
                _problem(
                    "sr-demo-p-aa-pi-2",
                    "To precipitate a protein out of solution you tune the buffer "
                    "to its pI. Why does it aggregate there?",
                    [
                        "It is most soluble at its pI",
                        "With zero net charge it stops repelling its neighbors",
                        "The pI denatures the protein",
                        "The pI maximizes the protein's charge",
                    ],
                    1,
                ),
            ],
        ),
    ],
)

_AA_METABOLISM = _leaf(
    "sr-demo-aa-metabolism",
    "Metabolism",
    [
        _concept(
            "sr-demo-c-aa-metabolism",
            "Amino-acid catabolism is nitrogen disposal",
            "Amino-acid catabolism is fundamentally nitrogen handling: the "
            "alpha-amino group is first collected onto glutamate by transamination, "
            "then released as ammonium by oxidative deamination so it can be "
            "disposed of as urea, while the leftover carbon skeleton feeds central "
            "metabolism.",
            [
                _problem(
                    "sr-demo-p-aa-metabolism-1",
                    "Alanine aminotransferase moves alanine's amino group onto "
                    "alpha-ketoglutarate, leaving pyruvate and glutamate. What has "
                    "happened to the nitrogen?",
                    [
                        "It has already been excreted as urea",
                        "It has only changed carriers (onto glutamate), not left",
                        "It has been oxidized to nitrate",
                        "It has been stored as a new amino acid",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-aa-metabolism-2",
                    "Glutamate dehydrogenase then removes glutamate's amino group "
                    "as free ammonium. What does this deamination accomplish?",
                    [
                        "It stores nitrogen for later",
                        "It liberates nitrogen for packaging into urea",
                        "It makes ATP directly from the amino group",
                        "It builds a nonpolar side chain",
                    ],
                    1,
                ),
            ],
        ),
    ],
)

# --- Proteins -------------------------------------------------------------

_PROTEIN_LEVELS = _leaf(
    "sr-demo-prot-levels",
    "Levels of structure",
    [
        _concept(
            "sr-demo-c-prot-levels",
            "Each level is defined by its interaction",
            "The levels of protein structure are defined by the kind of "
            "interaction at each tier, not by size: primary is the covalent "
            "sequence; secondary is local backbone hydrogen bonding (alpha-helices, "
            "beta-sheets); tertiary is one chain's three-dimensional fold from "
            "side-chain interactions; quaternary is the assembly of multiple chains.",
            [
                _problem(
                    "sr-demo-p-prot-levels-1",
                    "A mutation disrupts the regularly spaced backbone hydrogen "
                    "bonds of an alpha-helix; the sequence and every side chain are "
                    "unchanged. Which level is affected?",
                    ["Primary", "Secondary", "Tertiary", "Quaternary"],
                    1,
                ),
                _problem(
                    "sr-demo-p-prot-levels-2",
                    "Each subunit of a four-subunit protein folds normally, but "
                    "weakened contacts where the subunits meet let it fall apart "
                    "into monomers. Which level?",
                    ["Secondary", "Tertiary", "Quaternary", "Primary"],
                    2,
                ),
            ],
        ),
    ],
)

_PROTEIN_FOLDING = _leaf(
    "sr-demo-prot-folding",
    "Folding & denaturation",
    [
        _concept(
            "sr-demo-c-prot-folding",
            "Denaturation breaks the weak forces",
            "Folding and denaturation turn on the weak, non-covalent forces (the "
            "hydrophobic effect, hydrogen bonds, van der Waals and ionic contacts) "
            "that hold a chain in its native fold. A denaturant disrupts those "
            "forces and unfolds the protein while leaving the primary sequence "
            "intact; whether function returns is just whether the chain re-finds "
            "the native state or gets trapped and aggregates.",
            [
                _problem(
                    "sr-demo-p-prot-folding-1",
                    "Adding 8 M urea abolishes a ribonuclease's activity; dialyzing "
                    "the urea away restores it, and the peptide bonds were never "
                    "touched. What happened?",
                    [
                        "The primary sequence was cut",
                        "Non-covalent forces were disrupted, then re-formed",
                        "The active-site residues were chemically modified",
                        "The enzyme was competitively inhibited",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-prot-folding-2",
                    "Heating the same enzyme past its melting point abolishes "
                    "activity and the chains aggregate irreversibly. Why can heat "
                    "denaturation fail to reverse when urea's did?",
                    [
                        "Heat breaks the peptide bonds",
                        "Unfolded chains collide and aggregate instead of refolding",
                        "Heat rewrites the amino-acid sequence",
                        "Heat attaches phosphate groups to the chain",
                    ],
                    1,
                ),
            ],
        ),
    ],
)

# --- Enzymes --------------------------------------------------------------

_ENZYME_KINETICS = _leaf(
    "sr-demo-enz-kinetics",
    "Kinetics",
    [
        _concept(
            "sr-demo-c-enz-kinetics",
            "Michaelis-Menten: Vmax and Km",
            "Michaelis-Menten kinetics describes enzyme rate as a saturable, "
            "hyperbolic function of substrate concentration with two constants. "
            "Vmax is the ceiling reached when every active site is occupied, and "
            "Km, the substrate concentration giving half of Vmax, is an inverse "
            "measure of affinity (a lower Km means tighter binding).",
            [
                _problem(
                    "sr-demo-p-enz-kinetics-1",
                    "An enzyme's initial velocity climbs with substrate, then bends "
                    "over to a plateau. What does the plateau represent?",
                    [
                        "Km",
                        "Vmax, with every active site occupied",
                        "The inhibitor constant Ki",
                        "The isoelectric point",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-enz-kinetics-2",
                    "Enzyme X reaches half its maximum rate at 0.1 mM substrate; "
                    "enzyme Y needs 2 mM. Which binds substrate more tightly?",
                    [
                        "Enzyme Y, it needs more substrate",
                        "Enzyme X, it has the lower Km",
                        "They bind equally tightly",
                        "You cannot tell without Vmax",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-enz-kinetics-3",
                    "Two enzymes share a substrate. Reading only Km, what does a "
                    "lower Km tell you?",
                    [
                        "Weaker substrate binding",
                        "A higher Vmax",
                        "Tighter substrate binding (higher affinity)",
                        "More inhibitor is present",
                    ],
                    2,
                ),
            ],
        ),
    ],
)

_ENZYME_INHIBITION = _leaf(
    "sr-demo-enz-inhibition",
    "Inhibition",
    [
        _concept(
            "sr-demo-c-enz-inhibition",
            "Read the inhibitor type off Km and Vmax",
            "A reversible inhibitor is identified by the pattern it leaves in Km "
            "and Vmax. Apparent Km up with Vmax preserved, and rescue by excess "
            "substrate, means the inhibitor competes at the active site "
            "(competitive). Vmax down with Km preserved, and no rescue, means it "
            "binds elsewhere (noncompetitive).",
            [
                _problem(
                    "sr-demo-p-enz-inhibition-1",
                    "With a drug present, Vmax is unchanged but apparent Km doubles, "
                    "and flooding with substrate fully restores the rate. What kind "
                    "of inhibition is this?",
                    [
                        "Noncompetitive",
                        "Competitive",
                        "Irreversible",
                        "No inhibition at all",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-enz-inhibition-2",
                    "With a compound present, Vmax falls while apparent Km is "
                    "unchanged, and excess substrate does not restore the rate. "
                    "What kind of inhibition?",
                    [
                        "Competitive",
                        "Noncompetitive",
                        "The compound is just extra substrate",
                        "Allosteric activation",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-enz-inhibition-3",
                    "For a competitive inhibitor, apparent Km = Km * (1 + [I]/Ki). "
                    "If the apparent Km doubles, what is [I]?",
                    ["2 * Ki", "Ki", "Ki / 2", "Zero"],
                    1,
                ),
            ],
        ),
    ],
)

_ENZYME_REGULATION = _leaf(
    "sr-demo-enz-regulation",
    "Regulation",
    [
        _concept(
            "sr-demo-c-enz-regulation",
            "The cell tunes its own enzymes",
            "Enzyme regulation is the cell deliberately tuning an enzyme's activity "
            "to match its needs, distinct from an outside molecule merely competing "
            "for the active site: allosteric effectors at regulatory sites (often "
            "feedback inhibition of a committed step), cooperativity, and covalent "
            "modification such as phosphorylation.",
            [
                _problem(
                    "sr-demo-p-enz-regulation-1",
                    "A pathway's end product binds the first committed enzyme at a "
                    "site away from the active site, slowing it; as the product is "
                    "used up, the enzyme speeds back up. This is:",
                    [
                        "Competitive inhibition",
                        "Feedback (allosteric) regulation",
                        "Denaturation",
                        "Plain kinetics with no effector",
                    ],
                    1,
                ),
                _problem(
                    "sr-demo-p-enz-regulation-2",
                    "A kinase attaches a phosphate to switch an enzyme between "
                    "active and inactive forms, and a phosphatase removes it. This "
                    "control is:",
                    [
                        "Covalent modification (regulation)",
                        "Competitive inhibition",
                        "A change in the primary sequence",
                        "Simple substrate saturation",
                    ],
                    0,
                ),
            ],
        ),
    ],
)


def build_hierarchy(deck_id: str = authoring.NEW_DECK_ID) -> dict[str, Any]:
    """The complete demo hierarchy as an authoring-store blob.

    ``deck_id`` defaults to the sentinel the create flow uses, so
    :func:`anki.speedrun.authoring.save_hierarchy` creates the backing deck from
    the root title on first save.
    """
    root = _branch(
        "sr-demo-root",
        DEMO_DECK_NAME,
        [
            _branch(
                "sr-demo-aa",
                "Amino acids",
                [_AA_STRUCTURE, _AA_PKA, _AA_METABOLISM],
            ),
            _branch(
                "sr-demo-prot",
                "Proteins",
                [_PROTEIN_LEVELS, _PROTEIN_FOLDING],
            ),
            _branch(
                "sr-demo-enz",
                "Enzymes",
                [_ENZYME_KINETICS, _ENZYME_INHIBITION, _ENZYME_REGULATION],
            ),
        ],
    )
    return {"deckId": deck_id, "root": root}


def seed(col: anki.collection.Collection) -> bool:
    """Create the demo deck if it is not already present. Idempotent.

    Returns True when it seeded, False when the deck already existed (so a
    repeat call, e.g. on every collection load, is a no-op that never clobbers
    the user's edits or study state). Materializes the concepts into FSRS cards
    so the deck shows a To Do count and is studyable right away.
    """
    if col.decks.id_for_name(DEMO_DECK_NAME) is not None:
        return False

    saved = authoring.save_hierarchy(col, build_hierarchy())
    deck_id = saved.get("deckId") or ""
    if not deck_id:
        return False

    materialize.reconcile(col, deck_id)
    return True
