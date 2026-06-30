# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Hand-authored Learn seed for the biochemistry foundation (decision D21).

Pure data, no ``anki`` import. A small but complete slice (spec-study-model
section 9): contrasting-case concept cards and principle-first scaffold items
covering every leaf topic of the biomolecules foundation (amino acids,
proteins, enzymes) from spec-topic-taxonomy. Each leaf topic carries at least
one ``ContrastingCase`` and one ``ApplicationItem`` so it can be driven the
whole way up the mastery progression, concept -> application (spec-mastery-
progression section 2). Held to the section 10 quality bar: the two cases in a
pair share one deep structure under different surfaces, scaffold distractors are
real confusable siblings, and every cue names the discriminating feature.
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


# --- Contrasting-case concept cards (one+ per leaf topic) ------------------

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

CASE_AMINO_ACID_STRUCTURE = ContrastingCase(
    topic_id=TOPIC_AA_STRUCTURE,
    case_a=(
        "A structural biologist finds a residue buried in the water-excluding "
        "core of a folded globular protein. From its location alone she predicts "
        "its side chain is a nonpolar hydrocarbon (leucine, valine, "
        "phenylalanine) rather than a charged one, and the crystal structure "
        "confirms it."
    ),
    case_b=(
        "A protein engineer needs to place a residue on a loop that sits in the "
        "aqueous solvent and hydrogen-bonds with water. She rules out isoleucine "
        "and phenylalanine and chooses a polar residue such as serine or "
        "glutamine, because only those make favourable contacts with water."
    ),
    similarity_prompt=(
        "One reads a side chain off where a residue sits, the other places a "
        "residue by what its side chain needs. What single property are both "
        "using?"
    ),
    concept=(
        "An amino acid's behaviour is set by its side chain (R group), not the "
        "backbone every residue shares. Classifying the R group as nonpolar, "
        "polar uncharged, acidic, or basic predicts where the residue partitions "
        "and what it can bond: hydrophobic side chains bury in the nonpolar core, "
        "polar and charged ones face the water. Reading a residue's location and "
        "choosing a residue for a location are the same rule run forwards and "
        "backwards; the specific protein is just surface."
    ),
)

CASE_AMINO_ACID_PI = ContrastingCase(
    topic_id=TOPIC_AA_PKA_TITRATION,
    case_a=(
        "At pH 7.6 a mixture of amino acids is placed in an electric field. "
        "Lysine (pI ~9.7) drifts toward the cathode, aspartate (pI ~2.8) toward "
        "the anode, and histidine (pI ~7.6) barely moves, because the running pH "
        "sits right at its isoelectric point."
    ),
    case_b=(
        "To precipitate a protein out of solution, a biochemist tunes the buffer "
        "to the protein's isoelectric point, where it carries no net charge, "
        "stops repelling its neighbours, and aggregates. Shifting the pH away "
        "from pI in either direction redissolves it."
    ),
    similarity_prompt=(
        "Electrophoretic direction and minimum solubility are predicted by the "
        "same single comparison. What is it?"
    ),
    concept=(
        "Net charge is fixed by where the ambient pH sits relative to a "
        "molecule's pKa values, summarised by the isoelectric point (pI): the pH "
        "at which positive and negative charges balance to zero net charge. "
        "Below pI the species is net positive (migrates to the cathode, stays "
        "soluble), above pI it is net negative (migrates to the anode), and at "
        "pI it is neutral and least soluble. Electrophoretic migration and "
        "isoelectric precipitation are one rule, pH versus pI, seen two ways."
    ),
)

CASE_AMINO_ACID_METABOLISM = ContrastingCase(
    topic_id=TOPIC_AA_METABOLISM,
    case_a=(
        "In a reconstituted assay, alanine aminotransferase moves the "
        "alpha-amino group of alanine onto alpha-ketoglutarate, leaving pyruvate "
        "(a carbon skeleton bound for energy metabolism) and collecting the "
        "nitrogen as glutamate. No nitrogen has left the system yet; it has only "
        "changed carriers."
    ),
    case_b=(
        "In a second assay, glutamate dehydrogenase takes that glutamate and "
        "oxidatively removes the amino group as free ammonium, regenerating "
        "alpha-ketoglutarate. Now the nitrogen is liberated as the toxic "
        "ammonium the urea cycle will package for excretion."
    ),
    similarity_prompt=(
        "Two enzymes act on two different molecules, yet they are consecutive "
        "moves in one strategy. What is the shared underlying purpose?"
    ),
    concept=(
        "Amino-acid catabolism is fundamentally nitrogen handling: the "
        "alpha-amino group is first collected onto glutamate by transamination, "
        "then released as ammonium by oxidative deamination, so it can be "
        "disposed of as urea while the leftover carbon skeleton feeds central "
        "metabolism. Transamination and deamination look like different "
        "reactions but are the same deep step, separating nitrogen from carbon "
        "and routing the nitrogen toward excretion. The enzyme and carrier are "
        "surface; nitrogen disposal is the invariant."
    ),
)

CASE_PROTEIN_STRUCTURE_LEVELS = ContrastingCase(
    topic_id=TOPIC_PROTEIN_STRUCTURE_LEVELS,
    case_a=(
        "A mutation disrupts the regularly spaced backbone hydrogen bonds that "
        "hold a stretch of a polypeptide in an alpha-helix, while every side "
        "chain and the chain's sequence are unchanged. Asked which level of "
        "organisation is hit, you answer secondary structure."
    ),
    case_b=(
        "A different defect leaves each subunit of a four-subunit protein folded "
        "normally but weakens the contacts where the subunits meet, so the "
        "assembled tetramer falls apart into monomers. Asked the level, you "
        "answer quaternary structure."
    ),
    similarity_prompt=(
        "Both questions name a level of protein structure from a single clue. "
        "What kind of clue are both keying on?"
    ),
    concept=(
        "The levels of protein structure are defined by the kind of interaction "
        "at each tier, not by size: primary is the covalent sequence of "
        "residues; secondary is local backbone hydrogen bonding (alpha-helices, "
        "beta-sheets); tertiary is the three-dimensional fold of one chain set "
        "by side-chain interactions; quaternary is the assembly of multiple "
        "chains. Identifying a level means matching the disturbed interaction to "
        "its tier, backbone H-bonds to secondary, subunit contacts to "
        "quaternary, whatever the protein."
    ),
)

CASE_PROTEIN_FOLDING = ContrastingCase(
    topic_id=TOPIC_PROTEIN_FOLDING,
    case_a=(
        "Adding 8 M urea to a solution of a folded ribonuclease abolishes its "
        "activity; when the urea is dialysed away the enzyme spontaneously "
        "refolds and activity returns. The peptide bonds were never touched."
    ),
    case_b=(
        "Gently heating the same kind of enzyme past its melting temperature "
        "also abolishes activity, but as the unfolded chains collide they "
        "aggregate irreversibly into a precipitate that no longer refolds on "
        "cooling."
    ),
    similarity_prompt=(
        "A chaotrope and a temperature change cause the same loss of function by "
        "the same route, differing only in whether it reverses. What is that "
        "shared route?"
    ),
    concept=(
        "Folding and denaturation turn on the weak, non-covalent forces, the "
        "hydrophobic effect, hydrogen bonds, van der Waals and ionic contacts, "
        "that hold a chain in its native, lowest-free-energy fold. Any "
        "denaturant that disrupts those forces (urea, heat, extreme pH, "
        "detergents) unfolds the protein while leaving its primary sequence "
        "intact; whether function returns is just whether the chain re-finds the "
        "native minimum or gets trapped and aggregates. The agent is surface; "
        "disrupting the stabilising interactions is the concept."
    ),
)

CASE_ENZYME_KINETICS = ContrastingCase(
    topic_id=TOPIC_ENZYME_KINETICS,
    case_a=(
        "You plot an enzyme's initial velocity against substrate concentration: "
        "the rate climbs steeply while substrate is scarce, then bends over to a "
        "plateau where adding more substrate barely changes it. You read the "
        "plateau as the enzyme's maximum velocity."
    ),
    case_b=(
        "Two enzymes act on the same substrate. Enzyme X reaches half its "
        "maximum rate at 0.1 mM substrate; enzyme Y needs 2 mM to reach half of "
        "its maximum. You conclude enzyme X binds the substrate more tightly."
    ),
    similarity_prompt=(
        "One case reads a single saturation curve, the other compares two "
        "enzymes by a single number. What shared framework are both using?"
    ),
    concept=(
        "Michaelis-Menten kinetics describes enzyme rate as a saturable, "
        "hyperbolic function of substrate concentration with two constants. Vmax "
        "is the ceiling reached when every active site is occupied, and Km, the "
        "substrate concentration giving half of Vmax, is an inverse measure of "
        "affinity (lower Km means tighter binding). Reading a plateau as Vmax and "
        "comparing half-maximal concentrations as affinity are the same "
        "framework, the shape of the saturation curve, applied to one enzyme or "
        "to two."
    ),
)

CASE_ENZYME_REGULATION = ContrastingCase(
    topic_id=TOPIC_ENZYME_REGULATION,
    case_a=(
        "The end product of a biosynthetic pathway accumulates and binds the "
        "pathway's first committed enzyme at a regulatory site away from the "
        "active site, lowering its activity; as the product is consumed and its "
        "level drops, the enzyme speeds back up. The pathway throttles itself to "
        "demand."
    ),
    case_b=(
        "A hormone cascade has a kinase attach a phosphate to a metabolic "
        "enzyme, flipping it between active and inactive forms; a phosphatase "
        "later removes the phosphate to flip it back. The enzyme's output tracks "
        "the hormonal signal."
    ),
    similarity_prompt=(
        "Feedback by a small molecule and a covalent phosphate look nothing "
        "alike, yet they serve one function. What is it?"
    ),
    concept=(
        "Enzyme regulation is the cell deliberately tuning an enzyme's activity "
        "to match its needs, through mechanisms distinct from an outside "
        "molecule merely competing for the active site: allosteric effectors at "
        "regulatory sites (often feedback inhibition of a committed step), "
        "cooperativity, and covalent modification such as phosphorylation. A "
        "feedback metabolite and a phosphorylation are both the cell's own "
        "control signals adjusting flux; the mechanism is surface, integrated "
        "control of activity is the concept."
    ),
)

CONTRASTING_CASES: list[ContrastingCase] = [
    CASE_AMINO_ACID_STRUCTURE,
    CASE_AMINO_ACID_PKA,
    CASE_AMINO_ACID_PI,
    CASE_AMINO_ACID_METABOLISM,
    CASE_PROTEIN_STRUCTURE_LEVELS,
    CASE_PROTEIN_FOLDING,
    CASE_ENZYME_KINETICS,
    CASE_ENZYME_INHIBITION,
    CASE_ENZYME_REGULATION,
]


# --- Principle-first scaffold items (one+ per leaf topic) ------------------

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

APPLICATION_AMINO_ACID_STRUCTURE = ApplicationItem(
    problem_id="speedrun-app-amino-acid-structure",
    problem=(
        "A residue's side chain is a plain branched hydrocarbon with no "
        "heteroatoms and no ionizable group (for example the -CH2CH(CH3)2 of "
        "leucine). You must decide whether it tends to the protein's interior or "
        "its solvent-exposed surface. First classify what kind of problem this "
        "is."
    ),
    solution=(
        "The decisive fact is the side chain's chemistry: a pure hydrocarbon R "
        "group with no polar or ionizable atoms is nonpolar, so the hydrophobic "
        "effect drives it into the water-excluding interior, away from solvent. "
        "No pH, charge, or reaction is involved; you classify the R group and "
        "read off the consequence. That is amino-acid structure and "
        "classification."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_AMINO_ACIDS,
        TOPIC_AA_STRUCTURE,
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
            TOPIC_AA_STRUCTURE,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_PROTEINS: (
            "A residue only sits 'interior vs surface' once a chain has folded, "
            "but the answer is fixed by the residue's own R-group chemistry, not "
            "by secondary or tertiary architecture. That is amino-acid "
            "structure, not the proteins category."
        ),
        CATEGORY_ENZYMES: (
            "Nothing here is catalytic: no substrate, rate, or active site. You "
            "are classifying an amino acid by its side chain, not analysing an "
            "enzyme."
        ),
        TOPIC_AA_PKA_TITRATION: (
            "There is no ionizable group and no pH in play, so protonation and "
            "buffering never enter. The side chain is a neutral hydrocarbon; "
            "classifying it is the structure topic, not pKa/titration."
        ),
        TOPIC_AA_METABOLISM: (
            "The residue is not being made or broken down, and no bond in it is "
            "transformed. You only classify its side chain to predict placement, "
            "which is structure, not metabolism."
        ),
    },
)

APPLICATION_AMINO_ACID_METABOLISM = ApplicationItem(
    problem_id="speedrun-app-amino-acid-metabolism",
    problem=(
        "In a liver extract, a labelled amino acid's alpha-amino nitrogen is "
        "followed as it is transferred to alpha-ketoglutarate and then released "
        "as ammonium that is built into urea, while the leftover carbon skeleton "
        "emerges as a citric-acid-cycle intermediate. You must decide what kind "
        "of problem governs the fate of that nitrogen. First classify it."
    ),
    solution=(
        "Following the amino group through transamination (onto glutamate via "
        "alpha-ketoglutarate) and then deamination into the urea cycle, while "
        "the carbon skeleton enters the citric acid cycle, is amino-acid "
        "catabolism: disposing of nitrogen and repurposing the carbon skeleton. "
        "It is not the residue's fixed structure or its charge at some pH; the "
        "amino acid is being chemically dismantled. That is the metabolism topic."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_AMINO_ACIDS,
        TOPIC_AA_METABOLISM,
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
            TOPIC_AA_METABOLISM,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_PROTEINS: (
            "The amino acid here is a free monomer being transformed, not part "
            "of a folded chain. No secondary or tertiary structure or folding is "
            "in question, so this is amino-acid chemistry, not the proteins "
            "category."
        ),
        CATEGORY_ENZYMES: (
            "Aminotransferases and dehydrogenases catalyse these steps, so "
            "'enzymes' is tempting, but the question is the fate of the amino "
            "acid (its nitrogen and carbon), not any enzyme's Km/Vmax. Classify "
            "by the molecule being transformed: amino acids."
        ),
        TOPIC_AA_STRUCTURE: (
            "Structure is the fixed skeleton, chirality, and R-group class of an "
            "intact residue. Here the amino acid is being taken apart (its amino "
            "group removed), a transformation, which is metabolism, not "
            "structural classification."
        ),
        TOPIC_AA_PKA_TITRATION: (
            "No pH or protonation state is at issue; the amino group is not "
            "gaining or losing a proton, it is enzymatically removed and "
            "disposed of. That is metabolism, not pKa/titration."
        ),
    },
)

APPLICATION_PROTEIN_STRUCTURE_LEVELS = ApplicationItem(
    problem_id="speedrun-app-protein-structure-levels",
    problem=(
        "A hemoglobin variant's alpha and beta subunits each translate in full "
        "and each folds into its normal shape, but an altered residue at the "
        "subunit interface weakens how the chains pack together, changing the "
        "assembled tetramer's cooperativity. You must say which structural level "
        "the defect lies in. First classify what kind of problem this is."
    ),
    solution=(
        "Each chain's sequence (primary) and individual fold (tertiary) are "
        "stated to be normal; what changed is the interface between separate "
        "chains, which is quaternary structure, the assembly of subunits. The "
        "question asks you to place the defect on the hierarchy of structural "
        "levels, with the interaction being subunit-subunit contact, so it is a "
        "levels-of-structure problem, not the folding process."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_PROTEINS,
        TOPIC_PROTEIN_STRUCTURE_LEVELS,
    ],
    steps=[
        ScaffoldStep("foundation", list(_FOUNDATION_OPTIONS), FOUNDATION_BIOMOLECULES),
        ScaffoldStep(
            "category",
            [CATEGORY_PROTEINS, CATEGORY_AMINO_ACIDS, CATEGORY_ENZYMES],
            CATEGORY_PROTEINS,
        ),
        ScaffoldStep(
            "topic",
            [TOPIC_PROTEIN_STRUCTURE_LEVELS, TOPIC_PROTEIN_FOLDING],
            TOPIC_PROTEIN_STRUCTURE_LEVELS,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_AMINO_ACIDS: (
            "The defect is in how whole chains assemble, not in one residue's "
            "side-chain chemistry. A single amino acid's properties don't "
            "capture subunit assembly; this is a protein-level question."
        ),
        CATEGORY_ENZYMES: (
            "Hemoglobin carries oxygen rather than catalysing a reaction, and "
            "there is no rate, substrate, or active site. The issue is "
            "structural organisation, so it sits in the proteins category, not "
            "enzymes."
        ),
        TOPIC_PROTEIN_FOLDING: (
            "Each subunit is stated to fold normally, so the folding pathway and "
            "denaturation are not the issue. The defect is which tier is "
            "affected (the subunit interface, i.e. quaternary), the "
            "levels-of-structure topic, not folding."
        ),
    },
)

APPLICATION_PROTEIN_FOLDING = ApplicationItem(
    problem_id="speedrun-app-protein-folding",
    problem=(
        "A purified enzyme is treated with urea plus a reducing agent that "
        "breaks its disulfide bonds; it loses all activity even though its "
        "amino-acid sequence is fully intact, and you are asked whether removing "
        "both agents could restore function. First classify what kind of problem "
        "this is."
    ),
    solution=(
        "Urea disrupts the non-covalent interactions and the reducing agent "
        "breaks the disulfide cross-links, so the chain unfolds: this is "
        "denaturation. The primary sequence is intact, so in principle the chain "
        "can refold and re-form its disulfides once both agents are removed, if "
        "it finds the native state rather than aggregating. The question is the "
        "unfolding and refolding process and its reversibility, the "
        "folding/denaturation topic."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_PROTEINS,
        TOPIC_PROTEIN_FOLDING,
    ],
    steps=[
        ScaffoldStep("foundation", list(_FOUNDATION_OPTIONS), FOUNDATION_BIOMOLECULES),
        ScaffoldStep(
            "category",
            [CATEGORY_PROTEINS, CATEGORY_AMINO_ACIDS, CATEGORY_ENZYMES],
            CATEGORY_PROTEINS,
        ),
        ScaffoldStep(
            "topic",
            [TOPIC_PROTEIN_STRUCTURE_LEVELS, TOPIC_PROTEIN_FOLDING],
            TOPIC_PROTEIN_FOLDING,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_AMINO_ACIDS: (
            "The sequence of residues is intact; what changed is the "
            "higher-order shape of the whole chain. One amino acid's chemistry "
            "doesn't capture unfolding, this is a protein-level phenomenon."
        ),
        CATEGORY_ENZYMES: (
            "It happens to be an enzyme, but no catalytic parameter (Km, Vmax, "
            "an inhibitor) is in play, only loss of the folded state. The handle "
            "is denaturation, so the category is proteins, not enzymes."
        ),
        TOPIC_PROTEIN_STRUCTURE_LEVELS: (
            "This is not asking which tier (primary to quaternary) the defect "
            "names; it asks whether a denaturing process reverses. Conditions "
            "that unfold and maybe refold a chain are the folding/denaturation "
            "topic, not the static levels-of-structure classification."
        ),
    },
)

APPLICATION_ENZYME_KINETICS = ApplicationItem(
    problem_id="speedrun-app-enzyme-kinetics",
    problem=(
        "An enzyme follows Michaelis-Menten behaviour. With no other molecule "
        "added, you measure its velocity across a range of substrate "
        "concentrations to find Vmax and the substrate concentration giving "
        "half-Vmax, planning to report its affinity for the substrate. Before "
        "computing, classify what kind of problem this is."
    ),
    solution=(
        "Characterising an enzyme by Vmax and Km from its velocity-versus-"
        "substrate data, with no inhibitor and no cellular effector involved, is "
        "pure enzyme kinetics. Km read at half-Vmax is the inverse-affinity "
        "number you want (lower Km means tighter binding). Because nothing is "
        "modifying the enzyme, it is kinetics in isolation, not inhibition or "
        "regulation."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_ENZYMES,
        TOPIC_ENZYME_KINETICS,
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
            TOPIC_ENZYME_KINETICS,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_AMINO_ACIDS: (
            "There are catalytic rate parameters (Vmax, Km) here, which only a "
            "working catalyst has; free amino acids have none. You are analysing "
            "an enzyme, not a single residue."
        ),
        CATEGORY_PROTEINS: (
            "Right macromolecule class, wrong handle: proteins is about "
            "architecture (folding, levels of structure). You are given rates "
            "and Km/Vmax, which is the enzyme's catalytic behaviour."
        ),
        TOPIC_ENZYME_INHIBITION: (
            "Inhibition needs an added molecule that shifts Km or Vmax; here "
            "nothing is added and you only measure the enzyme's own parameters. "
            "That is kinetics, not inhibition."
        ),
        TOPIC_ENZYME_REGULATION: (
            "Regulation is the cell tuning the enzyme (allosteric effectors, "
            "covalent modification, feedback). No such effector appears; you are "
            "measuring baseline Km/Vmax, which is plain kinetics."
        ),
    },
)

APPLICATION_ENZYME_INHIBITION_NONCOMPETITIVE = ApplicationItem(
    problem_id="speedrun-app-enzyme-inhibition-noncompetitive",
    problem=(
        "An enzyme obeys Michaelis-Menten kinetics. With a candidate compound "
        "present, the measured Vmax falls while the apparent Km is unchanged, "
        "and adding a large excess of substrate does not restore the original "
        "rate. Before you solve for Ki, classify what kind of problem this is."
    ),
    solution=(
        "Vmax decreased + Km unchanged + no rescue by excess substrate is the "
        "noncompetitive signature: the compound binds a site other than the "
        "active site (binding free enzyme and the enzyme-substrate complex "
        "alike), so flooding with substrate cannot outcompete it. It is still "
        "inhibition, identified by its Km/Vmax fingerprint, just the "
        "noncompetitive case rather than the competitive one."
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
            "catalyst has; free amino acids have none. You are reasoning about "
            "an enzyme."
        ),
        CATEGORY_PROTEINS: (
            "Right macromolecule class, wrong handle: proteins is about folding "
            "and levels of structure. Here you have rate parameters and an added "
            "compound, which is the enzyme's catalytic behaviour."
        ),
        TOPIC_ENZYME_KINETICS: (
            "Plain kinetics is the enzyme's Km/Vmax on its own. The decisive "
            "fact is an added molecule that changes those parameters, which is "
            "the inhibition sub-case, not kinetics in isolation."
        ),
        TOPIC_ENZYME_REGULATION: (
            "Regulation is the cell tuning its own enzyme (allosteric effectors, "
            "covalent modification, feedback). This is an exogenous compound "
            "judged by its effect on Km/Vmax in vitro, which is inhibition."
        ),
    },
)

APPLICATION_ENZYME_REGULATION = ApplicationItem(
    problem_id="speedrun-app-enzyme-regulation",
    problem=(
        "Phosphofructokinase-1 is slowed when ATP binds a regulatory site "
        "separate from its active site and is sped up when AMP binds, so the "
        "enzyme's rate tracks the cell's energy charge rather than its substrate "
        "level alone. You must classify how this enzyme's activity is being set. "
        "First classify what kind of problem this is."
    ),
    solution=(
        "ATP and AMP here are the cell's own energy-status signals binding a "
        "regulatory site (not the active site) to raise or lower flux through "
        "the enzyme: allosteric regulation matching activity to need. It is not "
        "an exogenous inhibitor read from a Km/Vmax fingerprint, nor a bare "
        "measurement of parameters; it is the cell controlling its enzyme. That "
        "is the regulation topic."
    ),
    correct_path=[
        FOUNDATION_BIOMOLECULES,
        CATEGORY_ENZYMES,
        TOPIC_ENZYME_REGULATION,
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
            TOPIC_ENZYME_REGULATION,
        ),
    ],
    feedback={
        **_FOUNDATION_FEEDBACK,
        CATEGORY_AMINO_ACIDS: (
            "The scenario is a catalyst's activity being modulated, which only "
            "an enzyme has; a free amino acid has no activity to regulate. This "
            "is the enzymes category."
        ),
        CATEGORY_PROTEINS: (
            "Right macromolecule class, wrong handle: proteins is about "
            "structure and folding. Here the issue is how a catalyst's activity "
            "is controlled, the enzyme's behaviour, not its architecture."
        ),
        TOPIC_ENZYME_KINETICS: (
            "Plain kinetics is the enzyme's Km/Vmax measured on its own. Here "
            "endogenous effectors (ATP, AMP) actively raise and lower activity, "
            "which is regulation, not kinetics in isolation."
        ),
        TOPIC_ENZYME_INHIBITION: (
            "Inhibition as a topic here is an outside molecule identified by how "
            "it shifts Km/Vmax in vitro. ATP and AMP are the cell's own energy "
            "signals tuning flux at a regulatory site, which is physiological "
            "regulation, not exogenous inhibition."
        ),
    },
)

APPLICATION_ITEMS: list[ApplicationItem] = [
    APPLICATION_AMINO_ACID_STRUCTURE,
    APPLICATION_AMINO_ACID_PKA,
    APPLICATION_AMINO_ACID_METABOLISM,
    APPLICATION_PROTEIN_STRUCTURE_LEVELS,
    APPLICATION_PROTEIN_FOLDING,
    APPLICATION_ENZYME_KINETICS,
    APPLICATION_ENZYME_INHIBITION,
    APPLICATION_ENZYME_INHIBITION_NONCOMPETITIVE,
    APPLICATION_ENZYME_REGULATION,
]
