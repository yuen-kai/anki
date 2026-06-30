# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Hand-authored Learn seed for the biochemistry foundation (decision D21).

Pure data, no ``anki`` import. A small, complete slice (spec-study-model
section 9): contrasting-case concept cards and principle-first scaffold items
for amino-acid and enzyme topics, all keyed to the taxonomy node ids from
spec-topic-taxonomy. Held to the section 10 quality bar: the two cases in a pair
share one deep structure under different surfaces, scaffold distractors are real
confusable siblings, and every cue names the discriminating feature.
"""

from __future__ import annotations

from dataclasses import dataclass

from anki.speedrun.feedback import ApplicationItem, ScaffoldStep

# Taxonomy node ids (spec-topic-taxonomy: mcat::<foundation>::<category>::<topic>).
# The biomolecules foundation is the only in-scope one; the two sibling
# foundations exist purely as honest level-1 distractors.
FOUNDATION_BIOMOLECULES = "mcat::biomolecules"
FOUNDATION_CELLULAR_SYSTEMS = "mcat::cellular_systems"
FOUNDATION_ORGAN_SYSTEMS = "mcat::organ_systems"

CATEGORY_AMINO_ACIDS = "mcat::biomolecules::amino_acids"
CATEGORY_PROTEINS = "mcat::biomolecules::proteins"
CATEGORY_ENZYMES = "mcat::biomolecules::enzymes"

TOPIC_AA_STRUCTURE = "mcat::biomolecules::amino_acids::structure"
TOPIC_AA_PKA_TITRATION = "mcat::biomolecules::amino_acids::pka_titration"
TOPIC_AA_METABOLISM = "mcat::biomolecules::amino_acids::metabolism"
TOPIC_PROTEIN_STRUCTURE_LEVELS = "mcat::biomolecules::proteins::structure_levels"
TOPIC_PROTEIN_FOLDING = "mcat::biomolecules::proteins::folding"
TOPIC_ENZYME_KINETICS = "mcat::biomolecules::enzymes::kinetics"
TOPIC_ENZYME_INHIBITION = "mcat::biomolecules::enzymes::inhibition"
TOPIC_ENZYME_REGULATION = "mcat::biomolecules::enzymes::regulation"

# Human labels for the chooser. The template shows these; node ids stay internal.
NODE_LABELS: dict[str, str] = {
    FOUNDATION_BIOMOLECULES: "Biomolecules: structure & function",
    FOUNDATION_CELLULAR_SYSTEMS: "Cells & cellular systems",
    FOUNDATION_ORGAN_SYSTEMS: "Organ systems & homeostasis",
    CATEGORY_AMINO_ACIDS: "Amino acids",
    CATEGORY_PROTEINS: "Proteins",
    CATEGORY_ENZYMES: "Enzymes",
    TOPIC_AA_STRUCTURE: "Structure & classification",
    TOPIC_AA_PKA_TITRATION: "pKa & titration",
    TOPIC_AA_METABOLISM: "Metabolism",
    TOPIC_PROTEIN_STRUCTURE_LEVELS: "Levels of structure",
    TOPIC_PROTEIN_FOLDING: "Folding & denaturation",
    TOPIC_ENZYME_KINETICS: "Kinetics",
    TOPIC_ENZYME_INHIBITION: "Inhibition",
    TOPIC_ENZYME_REGULATION: "Regulation",
}


def node_label(node_id: str) -> str:
    """Display label for a node id, falling back to a prettified last segment."""
    if node_id in NODE_LABELS:
        return NODE_LABELS[node_id]
    tail = node_id.rsplit("::", 1)[-1]
    return tail.replace("_", " ").capitalize()


@dataclass(frozen=True)
class ContrastingCase:
    """A SpeedrunConcept note: two worked cases with the same deep structure but
    different surfaces, a prompt to state the shared idea, then the concept.
    """

    topic_id: str
    case_a: str
    case_b: str
    similarity_prompt: str
    concept: str


# --- Contrasting-case concept cards (>= 2) ---------------------------------

CASE_ENZYME_INHIBITION = ContrastingCase(
    topic_id=TOPIC_ENZYME_INHIBITION,
    case_a=(
        "A pharmacologist titrates methotrexate into a fixed amount of "
        "dihydrofolate reductase and measures initial rates across substrate "
        "concentrations. On a Lineweaver-Burk plot the inhibited and "
        "uninhibited lines meet on the y-axis: Vmax is unchanged, but the "
        "apparent Km climbs as more drug is added. Flooding the assay with "
        "substrate fully restores the original rate."
    ),
    case_b=(
        "A toxicologist adds a heavy-metal ion to a dehydrogenase and runs the "
        "same rate series. Now the Lineweaver-Burk lines meet on the x-axis: "
        "the apparent Km is unchanged, but Vmax falls as more ion is added. "
        "Flooding the assay with substrate does not restore the original rate."
    ),
    similarity_prompt=(
        "Both experiments name an inhibitor's mechanism from nothing but how "
        "it moves the kinetic parameters. What single idea are they both using?"
    ),
    concept=(
        "A reversible inhibitor is identified by the pattern it leaves in Km "
        "and Vmax. Apparent Km up with Vmax preserved, and rescue by excess "
        "substrate, means the inhibitor competes at the active site "
        "(competitive). Vmax down with Km preserved, and no rescue, means it "
        "binds elsewhere (noncompetitive). The shared deep structure is that "
        "the shift in Km/Vmax is the fingerprint of where and how the inhibitor "
        "binds, the surface molecule (a drug vs a metal ion) is irrelevant."
    ),
)

CASE_AMINO_ACID_PKA = ContrastingCase(
    topic_id=TOPIC_AA_PKA_TITRATION,
    case_a=(
        "You titrate aspartic acid with NaOH and watch the pH. Three times the "
        "curve nearly flattens, large additions of base barely move the pH, and "
        "each flat zone sits right at one of aspartate's pKa values "
        "(about 2.0, 3.9, and 9.9). Between the flat zones, a tiny amount of "
        "base swings the pH sharply."
    ),
    case_b=(
        "A biochemist needs a side chain that can shuttle protons at "
        "physiological pH 7.4. Among aspartate (pKa ~3.9), histidine "
        "(pKa ~6.0), and lysine (pKa ~10.5) she chooses histidine, even though "
        "all three are ionizable, because only its pKa sits near 7. The others "
        "are almost fully protonated or deprotonated at 7.4 and barely budge."
    ),
    similarity_prompt=(
        "One case reads a titration curve, the other picks a residue for a job, "
        "yet both turn on the same single relationship. What is it?"
    ),
    concept=(
        "A weak acid or base resists pH change most strongly when the ambient "
        "pH is near its pKa, where it is about half protonated and the "
        "Henderson-Hasselbalch ratio [A-]/[HA] is ~1. The flat regions of the "
        "titration curve and the choice of histidine as the physiological "
        "buffer are the same fact seen from two sides: buffering capacity peaks "
        "at pH ~= pKa, and distance from pKa is exactly what makes a group a "
        "poor buffer and leaves it nearly fully protonated or deprotonated."
    ),
)

CONTRASTING_CASES: list[ContrastingCase] = [
    CASE_ENZYME_INHIBITION,
    CASE_AMINO_ACID_PKA,
]


# --- Principle-first scaffold items (>= 2) ---------------------------------

# Level-1 (foundation) distractors are shared: every biochem problem must first
# be told apart from the physiology/systems foundations.
_FOUNDATION_OPTIONS = [
    FOUNDATION_BIOMOLECULES,
    FOUNDATION_CELLULAR_SYSTEMS,
    FOUNDATION_ORGAN_SYSTEMS,
]

_FOUNDATION_FEEDBACK = {
    FOUNDATION_CELLULAR_SYSTEMS: (
        "This is one purified molecule measured in a tube, with no cell, "
        "membrane, or transport involved. That keeps it at the biomolecule "
        "level, not the cellular-systems level."
    ),
    FOUNDATION_ORGAN_SYSTEMS: (
        "There is no tissue, organ, or whole-body homeostasis here, just the "
        "behaviour of a single molecule. That is biomolecule chemistry, not an "
        "organ-system response."
    ),
}

APPLICATION_ENZYME_INHIBITION = ApplicationItem(
    problem_id="speedrun-app-enzyme-inhibition",
    problem=(
        "An enzyme obeys Michaelis-Menten kinetics. With a candidate drug "
        "present, the measured Vmax is unchanged but the apparent Km doubles, "
        "and adding a large excess of substrate fully restores the original "
        "uninhibited rate. Before you solve for Ki, classify what kind of "
        "problem this is."
    ),
    solution=(
        "Vmax unchanged + apparent Km increased + rescue by excess substrate is "
        "textbook competitive inhibition: the drug binds the active site and "
        "competes with substrate, so piling on substrate outcompetes it (Vmax "
        "preserved) while you need more substrate to reach half-Vmax (Km up). "
        "Quantify it from the shift, apparent Km = Km * (1 + [I]/Ki), so a "
        "doubling of Km means [I] = Ki."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_ENZYMES,
        TOPIC_ENZYME_INHIBITION,
    ],
    steps=[
        ScaffoldStep("foundation", list(_FOUNDATION_OPTIONS), FOUNDATION_BIOMOLECULES),
        ScaffoldStep(
            "category",
            [CATEGORY_ENZYMES, CATEGORY_AMINO_ACIDS, CATEGORY_PROTEINS],
            CATEGORY_ENZYMES,
        ),
        ScaffoldStep(
            "topic",
            [TOPIC_ENZYME_KINETICS, TOPIC_ENZYME_INHIBITION, TOPIC_ENZYME_REGULATION],
            TOPIC_ENZYME_INHIBITION,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_AMINO_ACIDS: (
            "The data are catalytic rates (Vmax, Km), which only a working "
            "catalyst has. Free amino acids have no Km, you are reasoning about "
            "an enzyme."
        ),
        CATEGORY_PROTEINS: (
            "Right macromolecule class, wrong handle: the proteins category is "
            "about what the molecule looks like (folding, levels of structure). "
            "Here you are given rate parameters and an inhibitor, so it is the "
            "enzyme's catalytic behaviour."
        ),
        TOPIC_ENZYME_KINETICS: (
            "Plain kinetics would be Km/Vmax of the enzyme on its own. The "
            "decisive fact is an added molecule that changes those parameters, "
            "which is the inhibition sub-case, not kinetics in isolation."
        ),
        TOPIC_ENZYME_REGULATION: (
            "Regulation is the cell tuning its own enzyme (allosteric effectors, "
            "covalent modification, feedback). This is an exogenous drug judged "
            "by its effect on Km/Vmax in vitro, which is inhibition."
        ),
    },
)

APPLICATION_AMINO_ACID_PKA = ApplicationItem(
    problem_id="speedrun-app-amino-acid-pka",
    problem=(
        "A glutamate residue (side-chain pKa ~4.1) sits on a protein surface in "
        "a buffer brought to pH 7.4. You must decide whether its side chain is "
        "protonated or charged, and how well it buffers at this pH. First "
        "classify what kind of problem this is."
    ),
    solution=(
        "At pH 7.4, well above the side-chain pKa of ~4.1, Henderson-Hasselbalch "
        "gives [A-]/[HA] ~= 10^(7.4 - 4.1) ~= 2000:1, so the carboxyl is "
        "essentially fully deprotonated and carries a negative charge (-COO-). "
        "Because 7.4 is far from its pKa, it buffers almost nothing here; it "
        "would buffer well only near pH 4. The whole problem is pKa/titration "
        "logic applied to one side chain."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_AMINO_ACIDS,
        TOPIC_AA_PKA_TITRATION,
    ],
    steps=[
        ScaffoldStep("foundation", list(_FOUNDATION_OPTIONS), FOUNDATION_BIOMOLECULES),
        ScaffoldStep(
            "category",
            [CATEGORY_AMINO_ACIDS, CATEGORY_PROTEINS, CATEGORY_ENZYMES],
            CATEGORY_AMINO_ACIDS,
        ),
        ScaffoldStep(
            "topic",
            [TOPIC_AA_STRUCTURE, TOPIC_AA_PKA_TITRATION, TOPIC_AA_METABOLISM],
            TOPIC_AA_PKA_TITRATION,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_PROTEINS: (
            "It is on a protein, but the question is not about secondary or "
            "tertiary structure or folding, it is whether one acidic group is "
            "protonated. That is amino-acid chemistry of the residue, not "
            "protein architecture."
        ),
        CATEGORY_ENZYMES: (
            "Nothing here is catalytic, there is no substrate, rate, or active "
            "site. You are reasoning about an amino acid's ionizable group, not "
            "an enzyme."
        ),
        TOPIC_AA_STRUCTURE: (
            "Structure covers the fixed skeleton (chirality, the backbone, "
            "classifying R-groups). The decisive variable here is protonation "
            "as a function of pH, which is the pKa/titration topic."
        ),
        TOPIC_AA_METABOLISM: (
            "Metabolism is how the residue is made or broken down "
            "(transamination, the urea cycle). This problem never transforms "
            "it, it only asks its charge at a given pH, which is pKa/titration."
        ),
    },
)

APPLICATION_ITEMS: list[ApplicationItem] = [
    APPLICATION_ENZYME_INHIBITION,
    APPLICATION_AMINO_ACID_PKA,
]
