# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

"""Deck data for :mod:`anki.speedrun.seed_mcat_ch31_35` (kept separate so the
seed module stays small).

Every concept title/content and every problem prompt/choice/answer is taken from
two Princeton Review MCAT books, not invented:

- Concept titles + content: the *MCAT Prep, 2021-2022* content-review chapters
  (ch 35-39), condensed from the book's section headings and prose.
- Problems: the *MCAT Workout, 5th Edition* (2024) chapters (ch 31-35),
  freestanding questions plus practice-passage questions (with the passage
  context folded into the prompt where needed); correct answers are taken from
  the book's solution keys (A=0, B=1, C=2, D=3). Two chapters fall short of 16
  faithfully text-renderable Workout questions and are topped up with real
  4-choice example MCQs from the matching Prep chapter rather than inventing
  anything: Ch 35 Thermodynamics (Workout has only 11 questions -> 5 from Prep
  ch 39) and Ch 33 Kinematics and Dynamics (4 Workout questions are
  figure/OCR-destroyed -> 2 from Prep ch 37). See the per-chapter notes below.

``HIERARCHY`` is the finished authoring-store blob (``{deckId, root}``); the
structural shape (5 leaves x 4 concepts x 4 problems) is asserted by the seed
tests.
"""

from __future__ import annotations

from typing import Any


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


DECK_NAME = "MCAT Ch 31–35: Organic Chemistry & Physics"


# =====================================================================
# Ch 31 - Organic Chemistry Reactions: Nucleophilic Substitution and Addition
#   Concepts: Prep 2021-2022, chapter 35 (sections 35.1-35.4)
#   Problems: Workout 2024, chapter 31 (freestanding Qs + practice passages)
# =====================================================================

_CH31 = _leaf(
    "sr-mcat3135-ch31",
    "Organic Chemistry Reactions: Nucleophilic Substitution and Addition",
    [
        _concept(
            "sr-mcat3135-ch31-c1",
            "Nucleophilic substitution: SN1 and SN2",
            "Nucleophilic substitution replaces a leaving group on an "
            "electrophilic carbon with a nucleophile, breaking one sigma bond to "
            "the leaving group as another forms to the nucleophile. The SN2 "
            "mechanism is concerted and bimolecular (rate = k[nucleophile]"
            "[electrophile]): the nucleophile attacks from the backside, giving "
            "complete inversion of configuration, and it is favored by "
            "less-substituted substrates (CH3 > 1deg > 2deg >> 3deg) and polar "
            "aprotic solvents. The SN1 mechanism is two-step and unimolecular "
            "(rate = k[electrophile]): the leaving group first dissociates to form "
            "a planar carbocation (the slow step), so it is favored by "
            "more-substituted substrates (3deg > 2deg >> 1deg) and protic "
            "solvents, and attack on either face of the carbocation gives "
            "racemization.",
            [
                _problem(
                    "sr-mcat3135-ch31-c1-p1",
                    "In Step 1 of a synthesis, Compound 1 (a nucleophile) reacts "
                    "with the primary alkyl halide 1-bromo-2-chloroethane "
                    "(BrCH2CH2Cl) to give Compound 2. What is the expected rate "
                    "law for Step 1?",
                    [
                        "rate = k[Compound 1]",
                        "rate = k[BrCH2CH2Cl]",
                        "rate = k[Compound 1][BrCH2CH2Cl]",
                        "rate = k[Compound 1]^2",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch31-c1-p2",
                    "In a multistep synthesis, a dihaloethane alkylates a "
                    "nucleophile in Step 1: one carbon-halogen bond must react in "
                    "Step 1 (so it needs a good leaving group), while the other "
                    "halogen must stay inert during a later basic step (Step 3) "
                    "yet react in a still-later substitution (Step 6). The passage "
                    "notes that 1,2-dichloroethane gave a lower yield than "
                    "1-bromo-2-chloroethane. Compared with 1-bromo-2-chloroethane, "
                    "which reagent would give a higher overall yield?",
                    [
                        "1,2-dibromoethane",
                        "1,2-dichloroethane",
                        "1-chloro-2-iodoethane",
                        "1-bromo-2-fluoroethane",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch31-c1-p3",
                    "A synthesis deprotonates Compound 1 (a phenol, pKa about 10) "
                    "with K2CO3 in Step 1, but deprotonates Compound 3 (a benzylic "
                    "alcohol, pKa about 16) with the stronger base KOH in Step 3. "
                    "Which best explains this choice of bases?",
                    [
                        "Compound 1 is a stronger acid than Compound 3, and K2CO3 "
                        "is a stronger base than KOH.",
                        "Compound 1 is a stronger acid than Compound 3, and KOH is "
                        "a stronger base than K2CO3.",
                        "Compound 3 is a stronger acid than Compound 1, and K2CO3 "
                        "is a stronger base than KOH.",
                        "Compound 3 is a stronger acid than Compound 1, and KOH is "
                        "a stronger base than K2CO3.",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch31-c1-p4",
                    "Compound J reacts with Compound K at 20degC to form two "
                    "products, L (20%) and M (80%). At 60degC, L and M form in 80% "
                    "and 20% yields. When the 20degC mixture is heated to 60degC "
                    "and left overnight, the composition slowly changes to 80% "
                    "L/20% M and then stays constant. Which statement is most "
                    "consistent with this?",
                    [
                        "An equilibrium mixture of L and M is obtained at 60degC.",
                        "The activation energy for formation of L is lower than "
                        "that for M.",
                        "M is more stable than L.",
                        "L is formed faster than M.",
                    ],
                    0,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch31-c2",
            "Enols, enolates, and alpha-proton acidity",
            "Protons on the carbon alpha to a carbonyl are unusually acidic "
            "because the carbanion left after deprotonation is resonance-"
            "stabilized as an enolate ion, with the negative charge delocalized "
            "onto the electronegative oxygen; the enolate is nucleophilic mainly "
            "at that alpha-carbon. A ketone and its enol are tautomers, readily "
            "interconvertible constitutional isomers that differ only in the "
            "position of a proton and a double bond, and the equilibrium normally "
            "lies far toward the keto form. Because the alpha-carbon is planar "
            "(sp2) in the enol/enolate, reprotonation can occur on either face, so "
            "a stereocenter at the alpha-carbon becomes racemized; a proton "
            "flanked by two carbonyl groups is especially acidic.",
            [
                _problem(
                    "sr-mcat3135-ch31-c2-p1",
                    "Glucose and fructose share metabolic pathways and can be "
                    "interconverted. Under which conditions could glucose be "
                    "converted to fructose?",
                    [
                        "An oxidizing environment",
                        "A reducing environment",
                        "An aqueous basic environment",
                        "An aqueous acidic environment",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch31-c2-p2",
                    "In the base-promoted self-Claisen condensation of ethyl "
                    "acetate, two mechanisms are proposed. In Mechanism I the first "
                    "step (deprotonation at the alpha-carbon) is rapid and "
                    "reversible, followed by a rate-determining second step; in "
                    "Mechanism II the first step is slow and irreversible. The "
                    "reaction is run in deuterium-labeled ethanol (CH3CH2OD), "
                    "quenched at 50% completion, and the recovered unreacted ethyl "
                    "acetate is analyzed by mass spectrometry. Which statement is "
                    "correct?",
                    [
                        "If Mechanism I is operative, recovered ethyl acetate will "
                        "contain only CH3CH2O-C(=O)-CH3.",
                        "If Mechanism I is operative, recovered ethyl acetate will "
                        "contain CH3CH2O-C(=O)-CH2D.",
                        "If Mechanism II is operative, recovered ethyl acetate will "
                        "contain CH3CHD-O-C(=O)-CH3.",
                        "If Mechanism II is operative, recovered ethyl acetate will "
                        "contain CH3CH2O-C(=O)-CH2D.",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch31-c2-p3",
                    "Base-promoted self-Claisen condensation uses optically active "
                    "Compound III (an ester whose only stereocenter is at the "
                    "alpha-carbon). Recall that in Mechanism I the first step, "
                    "deprotonation at the alpha-carbon, is rapid and reversible. "
                    "The reaction is quenched at 50% completion. Which statement is "
                    "most consistent with the passage?",
                    [
                        "If the reaction proceeds via Mechanism I, recovered "
                        "Compound III will be optically inactive.",
                        "If the reaction proceeds via Mechanism I, the chiral "
                        "center in recovered Compound III will have undergone "
                        "inversion of configuration.",
                        "If the reaction proceeds via Mechanism I, the self-Claisen "
                        "product of Compound III will be optically active.",
                        "If the reaction proceeds via Mechanism II, the chiral "
                        "center in recovered Compound III will have undergone "
                        "inversion of configuration.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch31-c2-p4",
                    "Compound I is the self-Claisen product of ethyl acetate (a "
                    "beta-ketoester, ethyl acetoacetate) whose enol form is "
                    "stabilized by intramolecular hydrogen bonding and is present "
                    "to a significant extent. Which infrared regions show "
                    "absorption for Compound I but NOT for ethyl acetate? "
                    "I. 3300-3500 cm-1  II. 1700-1750 cm-1  III. 1630-1650 cm-1",
                    [
                        "I only",
                        "I and III only",
                        "I and II only",
                        "I, II, and III",
                    ],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch31-c3",
            "Nucleophilic addition to aldehydes and ketones",
            "Because the carbonyl carbon is electrophilic, aldehydes and ketones "
            "undergo nucleophilic addition, in which a pi bond breaks and two "
            "sigma bonds form. Hydride reagents such as NaBH4 and LiAlH4 add "
            "hydride to reduce the carbonyl to an alcohol, and organometallic "
            "reagents such as Grignard reagents (R-MgX, used in an aprotic ether "
            "solvent) add as carbanion-like nucleophiles to give an alcohol after "
            "acidic workup. Other nucleophiles add similarly: cyanide gives "
            "cyanohydrins, alcohols give hemiacetals then acetals under acid "
            "catalysis, primary amines give imines, and secondary amines give "
            "enamines. The first step of the acid-catalyzed additions (acetal, "
            "imine, enamine) is protonation of the carbonyl oxygen, which makes "
            "the carbonyl carbon even more electrophilic.",
            [
                _problem(
                    "sr-mcat3135-ch31-c3-p1",
                    "The reaction of pyrrolidine (a secondary amine) with "
                    "cyclohexanone forms an enamine. The first step in this "
                    "reaction is:",
                    [
                        "protonation of the carbonyl oxygen.",
                        "attack by the amine nitrogen on the carbonyl carbon.",
                        "deprotonation of an alpha-hydrogen in cyclohexanone.",
                        "dehydration of a reaction intermediate.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch31-c3-p2",
                    "In a synthesis, 2-methylpropanal first undergoes a crossed "
                    "aldol addition with formaldehyde to give an aldehyde "
                    "(2,2-dimethyl-3-hydroxypropanal), which is then treated with "
                    "KCN. The product, Compound I, is a member of which class of "
                    "compounds?",
                    ["Nitrile", "Cyanohydrin", "Isocyanide", "Amide"],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch31-c3-p3",
                    "A solution of CH3CH2MgBr (0.09 mol) in diethyl ether is added "
                    "dropwise to a solution of methyl benzoate (0.1 mol) in diethyl "
                    "ether, then quenched with cold aqueous HCl and worked up. "
                    "Which best describes the crude reaction product?",
                    [
                        "1-Phenyl-1-propanol along with a small amount of unreacted "
                        "methyl benzoate",
                        "2-Phenyl-2-butanol, ethyl phenyl ketone, and unreacted "
                        "methyl benzoate",
                        "1-Phenyl-1-propanol and ethyl phenyl ketone",
                        "3-Phenyl-3-pentanol and unreacted methyl benzoate",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch31-c3-p4",
                    "Step 2 of a synthesis reduces an aldehyde to an alcohol. If "
                    "Step 2 is monitored by both IR spectroscopy and thin-layer "
                    "chromatography (TLC), which observations are expected?",
                    [
                        "Replacement of an IR peak at ~1700 cm-1 with a peak at "
                        "~3400 cm-1, and a TLC spot with a lower Rf than the "
                        "starting material.",
                        "Replacement of an IR peak at ~1700 cm-1 with a peak at "
                        "~3400 cm-1, and a TLC spot with a higher Rf than the "
                        "starting material.",
                        "Replacement of an IR peak at ~3400 cm-1 with a peak at "
                        "~1700 cm-1, and a TLC spot with a lower Rf than the "
                        "starting material.",
                        "Replacement of an IR peak at ~3400 cm-1 with a peak at "
                        "~1700 cm-1, and a TLC spot with a higher Rf than the "
                        "starting material.",
                    ],
                    0,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch31-c4",
            "Carboxylic acids and their derivatives",
            "Carboxylic acids and their derivatives (acid chlorides, anhydrides, "
            "esters, and amides) are electrophilic at the carbonyl carbon and "
            "react by nucleophilic addition-elimination (acyl substitution): a "
            "nucleophile adds to give a tetrahedral intermediate that then expels "
            "the electronegative leaving group. Reactivity decreases as the "
            "leaving group becomes more basic, giving the order acid chlorides > "
            "anhydrides > esters > amides. Esterification (a carboxylic acid plus "
            "an alcohol under acid catalysis), ester hydrolysis, and "
            "transesterification are reversible equilibria, and amides cannot be "
            "formed directly from a carboxylic acid and an amine because a fast "
            "acid-base reaction outcompetes the addition-elimination. Carboxylic "
            "acids with a carbonyl beta to the carboxylate (beta-keto acids) "
            "readily decarboxylate, losing CO2 through a cyclic transition state.",
            [
                _problem(
                    "sr-mcat3135-ch31-c4-p1",
                    "The acid-catalyzed reaction between methyl benzoate and "
                    "propanol is a(n):",
                    [
                        "reversible reaction, producing propyl benzoate and "
                        "methanol.",
                        "reversible reaction, producing methyl propanoate and "
                        "benzyl alcohol.",
                        "irreversible reaction, producing propyl benzoate and "
                        "methanol.",
                        "irreversible reaction, producing methyl propanoate and "
                        "benzyl alcohol.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch31-c4-p2",
                    "Testosterone cypionate is an injectable slow-release steroid. "
                    "The prodrug can be converted to testosterone and cypionic acid "
                    "in muscle tissue by which type of reaction?",
                    ["Reduction", "Retro aldol", "Decarboxylation", "Hydrolysis"],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch31-c4-p3",
                    "The reaction between cyclohex-3-ene-1-carboxylic acid and "
                    "dimethylamine does not form the desired amide. Which best "
                    "explains this observation?",
                    [
                        "The intermediate formed in the reaction preferentially "
                        "loses N(CH3)2 to regenerate starting materials.",
                        "Proton transfer reactions are very fast.",
                        "Dimethylamine is a gas at standard temperature and "
                        "pressure.",
                        "Amides can be synthesized from acid chlorides and amines.",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch31-c4-p4",
                    "In the citric acid cycle, isocitrate is converted to "
                    "alpha-ketoglutarate and carbon dioxide in a two-step, "
                    "enzyme-mediated process (isocitrate dehydrogenase). Which "
                    "statement most accurately describes this process?",
                    [
                        "Decarboxylation followed by reaction with NADH (coenzyme)",
                        "Decarboxylation followed by reaction with NAD+ (coenzyme)",
                        "Reaction with NADH (coenzyme) followed by decarboxylation",
                        "Reaction with NAD+ (coenzyme) followed by decarboxylation",
                    ],
                    3,
                ),
            ],
        ),
    ],
)


# =====================================================================
# Ch 32 - Biologically Important Molecules   (filled from subagent extraction)
# =====================================================================

_CH32 = _leaf(
    "sr-mcat3135-ch32",
    "Biologically Important Molecules",
    [
        _concept(
            "sr-mcat3135-ch32-c1",
            "Amino acids",
            "Proteins are built from twenty different amino acids, and the "
            "composition and sequence of those amino acids is what makes each "
            "protein unique. All twenty share the same nitrogen-carbon-carbon "
            "backbone; what distinguishes each one is its side chain (the variable "
            "R-group), which sets its physical and chemical properties. The "
            "alpha-carbon is a stereocenter, so every amino acid is chiral except "
            "glycine, whose alpha-carbon bears two hydrogens, and the amino acids "
            "found in animals have the L configuration.",
            [
                _problem(
                    "sr-mcat3135-ch32-c1-p1",
                    "At what pH would the net charge on a 0.3 M solution of the "
                    "amino acid asparagine (pKa alpha-COOH = 2.0, pKa alpha-NH3+ = "
                    "8.8) be minimized?",
                    ["2.0", "5.4", "8.8", "12.0"],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch32-c1-p2",
                    "An alpha-amino acid is prepared by subjecting an achiral "
                    "aldehyde (one containing no stereocenters) to the Strecker "
                    "synthesis: treatment with (1) NH4Cl and NaCN, then (2) H3O+. "
                    "The product(s) will be:",
                    ["enantiomers.", "optically active.", "regioisomers.", "amphipathic."],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch32-c1-p3",
                    "The Gabriel malonic ester synthesis prepares racemic "
                    "alpha-amino acids: a nucleophile is alkylated by an alkyl "
                    "halide (Compound X) in an SN2 reaction, and subsequent acid "
                    "hydrolysis and decarboxylation give the amino acid. Which of "
                    "the following can best be employed as Compound X?",
                    [
                        "Benzyl bromide",
                        "Bromobenzene",
                        "2-Bromo-2-methylpropane",
                        "2-Bromobutane",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch32-c1-p4",
                    "Gel electrophoresis is used to separate a mixture of arginine "
                    "(pI ~ 10.8), tryptophan (pI ~ 6.1), and aspartic acid (pI ~ "
                    "2.8). Which matrix pH could best separate the mixture into pure "
                    "components (each of the three carrying a different net charge)?",
                    ["pH 2.2", "pH 3.2", "pH 6.0", "pH 10.8"],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch32-c2",
            "Proteins and the peptide bond",
            "Polypeptides are formed by linking amino acids with peptide bonds; a "
            "peptide bond forms between the carboxyl group of one amino acid and "
            "the alpha-amino group of another with loss of water, and is simply an "
            "amide bond. An individual amino acid within the chain is called a "
            "residue, and the backbone runs from the amino terminus (made first) "
            "to the carboxy terminus (made last), so by convention the N-terminal "
            "residue is written first. The peptide bond is planar and rigid because "
            "resonance delocalizes the nitrogen's lone pair onto the carbonyl "
            "oxygen, giving the bond double-bond character and preventing rotation.",
            [
                _problem(
                    "sr-mcat3135-ch32-c2-p1",
                    "Partial hydrolysis of hexapeptide A gave four tripeptides (W, "
                    "X, Y, Z). Only one fragment migrated toward the negative "
                    "terminal in gel electrophoresis at pH 7. Which peptide is "
                    "hexapeptide A? (Single-letter codes: R = arginine, positive at "
                    "pH 7; D = aspartate and E = glutamate, negative at pH 7; I = "
                    "isoleucine and L = leucine, uncharged.)",
                    ["RIDDLE", "ELDDIR", "RILDDE", "EDIDLR"],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch32-c2-p2",
                    "UV absorbance at 280 nm is used to estimate protein "
                    "concentration, but the sample must be free of interfering "
                    "contaminants. If present in equimolar quantities, which "
                    "nonapeptide is least likely to affect the accuracy of the UV "
                    "measurement? (Aromatic residues W, Y, and F absorb near 280 nm; "
                    "W and Y absorb more strongly than F.)",
                    ["CYFQNCPRG", "CTFQNCPKG", "WAGGDASGE", "CYIQNCPLG"],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch32-c2-p3",
                    "A sample of the enzyme XYN I (19 kDa, pI 5.5) is contaminated "
                    "with Protein X (23 kDa, pI 7.5) and Protein Y (120 kDa, pI "
                    "5.4). Which procedure can best isolate pure XYN I?",
                    [
                        "Ion-exchange chromatography at pH 6.0, then size-exclusion "
                        "chromatography of the eluate",
                        "Size-exclusion chromatography, then SDS-PAGE of the eluate",
                        "Ion-exchange chromatography at pH 6.0, then gas-liquid "
                        "chromatography of the eluate",
                        "Size-exclusion chromatography, then gas-liquid "
                        "chromatography of the eluate",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch32-c2-p4",
                    "Diethylaminoethyl cellulose (DEAE-C) is a weak anion-exchange "
                    "resin whose protonated diethylamino group bears a positive "
                    "charge that binds negatively-charged proteins. Which statement "
                    "best describes a property of DEAE-C resin?",
                    [
                        "It is best used when the mobile-phase pH is buffered above "
                        "the pKa of the diethylamino groups.",
                        "Proteins bearing smaller charges are eluted more rapidly "
                        "than those bearing larger charges.",
                        "Binding of proteins is best performed using a high ionic "
                        "strength buffer.",
                        "It is a weak ion-exchange resin because of its low ability "
                        "to bind proteins.",
                    ],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch32-c3",
            "Carbohydrates",
            "Carbohydrates are chains of hydrated carbon atoms (formula CnH2nOn) "
            "that begin with an aldehyde or ketone and continue as a polyalcohol, "
            "and because their oxidation releases large amounts of energy they are "
            "the cell's principal energy source. A single unit is a monosaccharide; "
            "two joined form a disaccharide, and many form a polysaccharide. "
            "Monosaccharides are named as an aldose or ketose (for an aldehyde or "
            "ketone) plus a root for the number of carbons (triose, tetrose, "
            "pentose, hexose), and the sugars in our bodies have the D "
            "configuration.",
            [
                _problem(
                    "sr-mcat3135-ch32-c3-p1",
                    "Which of the following would result from treating "
                    "beta-D-galactopyranosyl-(1->4)-D-glucose (lactose) with NaBH4?",
                    [
                        "Reduction of C-1 of the glucose residue to yield an acyclic "
                        "primary alcohol",
                        "Reduction of C-1 of the glucose residue to yield a cyclic "
                        "ether",
                        "Cleavage of the glycosidic linkage to yield galactose and "
                        "glucose",
                        "No reaction, as all functional groups in lactose are stable "
                        "to the conditions",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch32-c3-p2",
                    "In the Maillard reaction, asparagine and a reducing sugar form "
                    "a Schiff base (an imine), which is then isomerized "
                    "(tautomerized) on the pathway to acrylamide. Which reagent "
                    "would perform this isomerization of the Schiff base?",
                    [
                        "H2SO4",
                        "NaOCH2CH3 (sodium ethoxide)",
                        "CH3MgBr (methylmagnesium bromide)",
                        "CH3COCH3 (acetone)",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch32-c3-p3",
                    "Which statement about Schiff base formation (a primary amine "
                    "plus a carbonyl forming an imine, releasing water) is true?",
                    [
                        "Under strongly acidic conditions, the rate of the reaction "
                        "increases.",
                        "Removing water from the system decreases the yield of the "
                        "Schiff base.",
                        "The loss of water is the thermodynamic driving force of the "
                        "reaction.",
                        "The reaction is under kinetic control, favoring the Schiff "
                        "base product.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch32-c3-p4",
                    "A protein's N-linked glycosylation attaches glycan chains to "
                    "asparagine residues. After digesting the protein into "
                    "fragments (some glycosylated, some not), which technique is "
                    "most appropriate to separate the glycopeptide fragments from "
                    "the non-glycosylated ones, given that lectins are "
                    "carbohydrate-binding proteins?",
                    [
                        "Size-exclusion chromatography only",
                        "Lectin affinity chromatography only",
                        "Size-exclusion and ion-exchange chromatography",
                        "Lectin affinity and ion-exchange chromatography",
                    ],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch32-c4",
            "Lipids",
            "Lipids are oily or fatty substances whose defining feature is "
            "hydrophobicity, since their nonpolar carbon-carbon and carbon-hydrogen "
            "bonds do not dissolve well in polar water. They fill three "
            "physiological roles: phospholipids form the cell-membrane barrier "
            "between the intracellular and extracellular space, triglycerides "
            "(fats) store energy in adipose cells, and cholesterol is the building "
            "block for the steroid hormones. Fatty acids are long unsubstituted "
            "alkanes ending in a carboxylic acid, typically 14 to 18 carbons; "
            "because they are built two carbons at a time from acetate, only "
            "even-numbered fatty acids are made in human cells, and a saturated "
            "fatty acid has no carbon-carbon double bonds.",
            [
                _problem(
                    "sr-mcat3135-ch32-c4-p1",
                    "Which of the following best characterizes the differences "
                    "between fatty acids and phospholipids?",
                    [
                        "Fatty acids have two tails and form micelles; "
                        "phospholipids have one tail and form membranes.",
                        "Fatty acids have two tails and form membranes; "
                        "phospholipids have two tails and form micelles.",
                        "Fatty acids have one tail and form micelles; phospholipids "
                        "have two tails and form membranes.",
                        "Fatty acids have one tail and form membranes; "
                        "phospholipids have two tails and form micelles.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch32-c4-p2",
                    "An ether solution contains a mixture of benzoic acid, phenol, "
                    "and aniline. Using diethyl ether as the organic phase and 5% "
                    "dilute aqueous acid or base as the aqueous phase, which "
                    "procedure could best separate the three?",
                    [
                        "Extract with 5% HCl, separate; then extract the ether layer "
                        "with 5% NaOH, separate.",
                        "Extract with 5% NaOH, separate; then extract the ether "
                        "layer with 5% HCl, separate.",
                        "Extract with 5% NaOH, separate; then extract the ether "
                        "layer with 5% NaHCO3, separate.",
                        "Extract with 5% NaHCO3, separate; then extract the ether "
                        "layer with 5% HCl, separate.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch32-c4-p3",
                    "A mixture of five amino acids (Asp, Phe, Met, Trp, Tyr) is "
                    "separated by reverse-phase HPLC with 10% aqueous methanol. In "
                    "reverse-phase HPLC the stationary phase is nonpolar, so more "
                    "hydrophobic species are retained longer. What is the expected "
                    "order of increasing retention time?",
                    [
                        "Met < Tyr < Asp < Phe < Trp",
                        "Trp < Phe < Asp < Met < Tyr",
                        "Asp < Tyr < Met < Phe < Trp",
                        "Trp < Phe < Met < Tyr < Asp",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch32-c4-p4",
                    "Three compounds are run on a silica gel TLC plate with 10% "
                    "ethyl acetate in hexane (normal-phase TLC: polar stationary "
                    "phase, relatively nonpolar mobile phase). How do the "
                    "retardation factors (Rf) change (i) if the stationary phase is "
                    "changed to the more polar alumina, and (ii) if the mobile phase "
                    "is changed to the more polar 25% ethyl acetate in hexane?",
                    [
                        "(i) all Rf increase; (ii) all Rf increase",
                        "(i) all Rf decrease; (ii) all Rf increase",
                        "(i) all Rf increase; (ii) all Rf decrease",
                        "(i) all Rf decrease; (ii) all Rf decrease",
                    ],
                    1,
                ),
            ],
        ),
    ],
)


# =====================================================================
# Ch 33 - Kinematics and Dynamics
#   Concepts: Prep 2021-2022, chapter 37 (sections 37.2/37.5/37.7/37.9).
#   Problems: Workout 2024, chapter 33. That chapter has 16 questions, but 4
#   depend on figures/graphs or have OCR-destroyed math choices (freestanding
#   Q1 projectile-time, Q4 round-trip graph, and one garbled choice each in the
#   others), leaving 14 faithfully recoverable. The remaining 2 are real
#   4-choice example MCQs from the matching Prep chapter 37 (Examples 37-54 and
#   37-55), verified against the book. Concepts were chosen to fit the available
#   problems (friction- and torque-heavy), which is why Friction and Torque are
#   used rather than Gravitation/Circular Motion.
# =====================================================================

_CH33 = _leaf(
    "sr-mcat3135-ch33",
    "Kinematics and Dynamics",
    [
        _concept(
            "sr-mcat3135-ch33-c1",
            "Kinematics",
            "Kinematics describes motion in terms of an object's position, "
            "velocity, and acceleration. Displacement is the change in position "
            "(final minus initial); it is a vector giving net distance plus "
            "direction and can be smaller than the total distance traveled. "
            "Average velocity is displacement divided by time (a vector), while "
            "speed is the magnitude of velocity, a scalar that is never negative. "
            "Average acceleration is the change in velocity divided by time, and "
            "because velocity is a vector, an object accelerates whenever its speed "
            "or its direction changes, so it can accelerate even while moving at "
            "constant speed.",
            [
                _problem(
                    "sr-mcat3135-ch33-c1-p1",
                    "Which of the following kinematic descriptions is physically "
                    "impossible?",
                    [
                        "An object in motion for a time dt has zero average velocity "
                        "but positive nonzero average speed.",
                        "An object maintains constant speed for a time dt but has "
                        "nonzero acceleration.",
                        "An object in motion for a time dt has zero displacement but "
                        "nonzero total distance.",
                        "An object in motion for a time dt has an average velocity "
                        "with a greater magnitude than its average speed.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch33-c1-p2",
                    "An object is projected from the ground at an angle of 30 "
                    "degrees above the horizontal and returns to the ground 8 "
                    "seconds later. What is the object's maximum height? (g = 10 "
                    "m/s^2)",
                    ["40 m", "80 m", "160 m", "There is not enough information to solve."],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch33-c1-p3",
                    "Two lead balls of unequal mass are thrown vertically on the "
                    "Moon. One is thrown directly upward, the other downward with "
                    "the same speed. Which statements is/are true? I. The two balls "
                    "reach the ground with the same speed. II. The two balls reach "
                    "the ground at the same time. III. The two balls are attracted "
                    "to the Moon with a force of equal magnitude.",
                    ["I only", "II only", "I and III only", "II and III only"],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch33-c1-p4",
                    "In a centrifuge, the particulate matter at the bottom of the "
                    "test tube is twice as far from the rotational axis as the fluid "
                    "at the top while the centrifuge runs at maximum speed. Given "
                    "that gravity is negligible under centrifugation, what is the "
                    "relationship between the accelerations of the liquid at the top "
                    "and the particles at the bottom?",
                    [
                        "a_liquid = a_particles",
                        "a_liquid = 2 a_particles",
                        "2 a_liquid = a_particles",
                        "The relationship is impossible to determine without knowing "
                        "the densities of the liquid and the particles.",
                    ],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch33-c2",
            "Mass, force, and Newton's laws",
            "Dynamics explains motion in terms of the forces acting on an object, "
            "where a force is a push or pull exerted by one object on another (such "
            "as gravity, the normal force, tension, or friction). Newton's First "
            "Law (the law of inertia) states that an object's velocity will not "
            "change unless a net force acts on it, so no net force means no "
            "acceleration; mass is the quantitative measure of an object's inertia. "
            "Newton's Second Law states that the net force equals mass times "
            "acceleration (F_net = ma), with the acceleration pointing in the "
            "direction of the net force, and the unit of force, 1 kg*m/s^2, is the "
            "newton. Newton's Third Law states that if one object exerts a force on "
            "a second, the second exerts an equal and opposite force on the first, "
            "and because these action-reaction forces act on different objects they "
            "do not cancel.",
            [
                _problem(
                    "sr-mcat3135-ch33-c2-p1",
                    "Which of the following best explains why one can drive a car "
                    "faster without skidding around a banked curved road than around "
                    "a level curved road?",
                    [
                        "The greater vertical component of normal force on the "
                        "banked curve increases the maximum frictional force, which "
                        "increases the maximum centripetal force.",
                        "The additional horizontal component of normal force on the "
                        "banked curve combines with friction from the road to "
                        "increase the maximum centripetal force.",
                        "The additional horizontal component of gravity on the "
                        "banked curve combines with friction from the road to "
                        "increase the maximum centripetal force.",
                        "The reduction of the gravitational force on the banked "
                        "curve means less centripetal force is required to keep the "
                        "car moving in a curved path than on a level road.",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch33-c2-p2",
                    "A block of mass M rests on an inclined plane at angle theta. A "
                    "cord attached to M runs up the plane, over a pulley at the top, "
                    "and supports a hanging block of mass m. The system is in static "
                    "equilibrium. What is the force of static friction between M and "
                    "the incline if M = 16 kg, m = 10 kg, theta = 30 degrees, mu_s = "
                    "0.4, and g = 10 m/s^2?",
                    [
                        "0 N",
                        "20 N down the plane",
                        "55 N up the plane",
                        "55 N down the plane",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch33-c2-p3",
                    "A nurse inserts a catheter into a patient. Under Thirion's Law, "
                    "the coefficient of friction of a rubber-like material decreases "
                    "as the normal force increases, and lubrication lowers the "
                    "coefficient of friction. Discomfort and shearing tissue damage "
                    "are caused by the force of friction, not merely by the "
                    "coefficient of friction. What should the nurse do to minimize "
                    "discomfort and shearing damage?",
                    [
                        "Squeeze harder and avoid lubricant to increase the "
                        "coefficient of friction between the catheter and the "
                        "tissue.",
                        "Squeeze harder and use lubricant to decrease the "
                        "coefficient of friction between the catheter and the "
                        "tissue.",
                        "Squeeze softer and avoid lubricant to increase the force of "
                        "friction between the catheter and the tissue.",
                        "Squeeze softer and use lubricant to decrease the force of "
                        "friction between the catheter and the tissue.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch33-c2-p4",
                    "At very high speeds, coefficients of friction are usually "
                    "reduced. When a copper pin slides against a copper surface, the "
                    "force required to keep the pin sliding at constant speed is most "
                    "likely:",
                    [
                        "zero.",
                        "greater at high speeds than at low speeds.",
                        "greater at low speeds than at high speeds.",
                        "equal at all speeds.",
                    ],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch33-c3",
            "Friction and contact forces",
            "When an object touches a surface, the surface exerts a contact force "
            "whose perpendicular component is the normal force; for an object "
            "resting on a flat horizontal surface with no other vertical forces, "
            "the normal force equals the object's weight (N = mg). Friction acts "
            "parallel to the surface and opposes relative sliding, and its "
            "magnitude is proportional to the normal force. Kinetic (sliding) "
            "friction has magnitude f_k = mu_k * N and points opposite the object's "
            "velocity, whereas static friction can take any value up to a maximum "
            "of f_s(max) = mu_s * N. For a given pair of surfaces the static "
            "coefficient is larger than the kinetic one, so it takes more force to "
            "start an object sliding than to keep it sliding.",
            [
                _problem(
                    "sr-mcat3135-ch33-c3-p1",
                    "A study measured static and kinetic coefficients of friction "
                    "for several material pairs. For silicone on skin, lubrication "
                    "lowered both coefficients, and the decrease in the static "
                    "coefficient was larger than the decrease in the kinetic one; "
                    "and the static coefficient for cast iron on cast iron was "
                    "greater than that for zinc on cast iron, while for kinetic "
                    "friction the reverse was true. Which can be validly concluded "
                    "from these data?",
                    [
                        "Lubrication has a more significant effect on the static "
                        "coefficient than on the kinetic coefficient.",
                        "If the static coefficient between two materials is greater "
                        "than that between two others, the kinetic coefficient will "
                        "always follow the same pattern.",
                        "The coefficient of kinetic friction is always less than or "
                        "equal to the coefficient of static friction.",
                        "None of the above can validly be concluded from the given "
                        "data.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch33-c3-p2",
                    "The coefficient of static friction between zinc and cast iron "
                    "is 0.85. A cast iron pot rests on a flat zinc surface. Which of "
                    "the following best approximates the steepest angle at which the "
                    "surface can be tilted before the pot starts to slide?",
                    ["40 degrees", "50 degrees", "60 degrees", "70 degrees"],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch33-c3-p3",
                    "For rubber on a rubber surface, Thirion's Law states that the "
                    "coefficient of friction decreases as the normal force "
                    "increases. The force required to initiate horizontal movement "
                    "(F') equals the maximum static friction (the static coefficient "
                    "times the weight). If such a rubber object has its mass "
                    "increased by adding weights inside it, which is true of the "
                    "force required to initiate movement (F') and the weight (w)?",
                    [
                        "The ratio of F' to w increases.",
                        "The ratio of F' to w remains constant.",
                        "The ratio of F' to w decreases.",
                        "The difference between F' and w remains constant.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch33-c3-p4",
                    "Polytetrafluoroethylene (PTFE, brand name Teflon) has a "
                    "coefficient of friction of about 0.04, far lower than any other "
                    "material listed in a table of coefficients. Which of the "
                    "following best explains why PTFE is often used to coat "
                    "non-stick cookware?",
                    [
                        "The toxicity of fumes given off by PTFE-coated pots is less "
                        "than that of standard cooking oils.",
                        "Its extremely low coefficient of friction reduces the "
                        "occurrence of food sticking to the pans.",
                        "Its extremely low coefficient of friction reduces the heat "
                        "lost to friction when food slides on PTFE surfaces.",
                        "PTFE was also used in the Manhattan Project to coat pipes "
                        "holding radioactive uranium compounds.",
                    ],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch33-c4",
            "Torque, center of mass, and equilibrium",
            "Torque is the measure of a force's effectiveness at making an object "
            "rotate about a pivot, given by torque = rF sin(theta), where r is the "
            "distance from the pivot to the point where the force is applied and "
            "theta is the angle between r and F; equivalently torque = (lever arm) "
            "x F. A force directed along a line through the pivot produces zero "
            "torque. The center of mass (or center of gravity) is the point where "
            "an object's mass can be treated as concentrated; for point masses "
            "along a line it is the mass-weighted average position. A system is in "
            "equilibrium when it has no net force (translational equilibrium) and "
            "no net torque (rotational equilibrium), so its velocity and rotation "
            "do not change.",
            [
                _problem(
                    "sr-mcat3135-ch33-c4-p1",
                    "The biceps tendon attaches about 3 cm from the elbow joint, "
                    "and the midpoint of the hand (where a dumbbell of weight w is "
                    "held) is 30 cm from the elbow. With the elbow bent at 90 "
                    "degrees, neglecting the weight of the forearm, and the dumbbell "
                    "lifted at a constant rate, what is the relation of the force "
                    "exerted by the biceps (F_biceps) to the weight of the dumbbell "
                    "(w)?",
                    [
                        "F_biceps = w",
                        "F_biceps = 10 w",
                        "10 F_biceps = w",
                        "14.1 F_biceps < w",
                    ],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch33-c4-p2",
                    "Amanda and Billy want to balance on a teeter-totter (a plank "
                    "on a fulcrum). They have a 3 m long, 10 kg plank and a boulder "
                    "as the fulcrum. Amanda weighs 650 N and Billy weighs 300 N. How "
                    "far from Amanda's seat should they place the fulcrum to balance "
                    "the plank and make the game as fair as possible?",
                    ["95 cm", "1 m", "1.5 m", "1.6 m"],
                    1,
                ),
                _problem(
                    "sr-mcat3135-ch33-c4-p3",
                    "Which of the following best explains why people with biceps "
                    "attachment points farther from their elbows tend to have "
                    "greater elbow flexion strength, and thus an improved ability to "
                    "perform a dumbbell curl?",
                    [
                        "An attachment point farther from the elbow increases the "
                        "force provided by muscle contraction.",
                        "An attachment point farther from the elbow decreases the "
                        "force provided by muscle contraction.",
                        "An attachment point farther from the elbow results in a "
                        "greater torque produced by the biceps as it contracts.",
                        "An attachment point closer to the hand results in a lesser "
                        "torque produced by the biceps as it contracts.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch33-c4-p4",
                    "A homogeneous rectangular metal sheet lies on a table and can "
                    "rotate about a vertical axis through its center. Four forces of "
                    "equal magnitude act on it, arranged so that they cancel in "
                    "pairs (two vertical forces point in opposite directions and two "
                    "horizontal forces point in opposite directions), yet each force "
                    "is offset from the center so that all four tend to rotate the "
                    "sheet in the same direction. Which statement is true?",
                    [
                        "The net force is zero, but the net torque is not.",
                        "The net torque is zero, but the net force is not.",
                        "Neither the net force nor the net torque is zero.",
                        "Both the net force and the net torque equal zero.",
                    ],
                    0,
                ),
            ],
        ),
    ],
)


# =====================================================================
# Ch 34 - Work and Energy   (filled from subagent extraction)
# =====================================================================

_CH34 = _leaf(
    "sr-mcat3135-ch34",
    "Work and Energy",
    [
        _concept(
            "sr-mcat3135-ch34-c1",
            "Work",
            "The work done by a constant force F acting through a displacement d "
            "is W = Fd cos(theta), where theta is the angle between F and d; only "
            "the component of the force parallel to the displacement does work, and "
            "when F points along d (theta = 0) this reduces to W = Fd. Work is a "
            "scalar measured in joules (one joule = one newton-meter) and can be "
            "positive, zero, or negative: positive when theta is acute, zero when "
            "theta = 90 degrees, and negative when theta is obtuse. The total work "
            "done on an object equals the sum of the work done by each force, which "
            "is the same as the work done by the net force.",
            [
                _problem(
                    "sr-mcat3135-ch34-c1-p1",
                    "Boxes of the same mass are lifted into the air either up a "
                    "frictionless ramp with force F_A, or with a pulley with force "
                    "F_B. Both boxes are lifted to a height of 3 meters. Which "
                    "method requires more work?",
                    [
                        "Equal work is required for both methods.",
                        "Greater work is required to lift the box with the pulley.",
                        "No work is required for either method.",
                        "Greater work is required to push the box up the ramp.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch34-c1-p2",
                    "An object of mass m is to be pushed up a frictionless inclined "
                    "plane of angle theta and height H. It can be pushed by a force "
                    "F_1 directed parallel to the surface of the incline, or by a "
                    "force F_2 directed horizontally. Which is true about the "
                    "minimum work required by each force and their mechanical "
                    "advantages?",
                    [
                        "W_1 < W_2 and MA_1 > MA_2",
                        "W_1 < W_2 and MA_1 = MA_2",
                        "W_1 = W_2 and MA_1 > MA_2",
                        "W_1 = W_2 and MA_1 = MA_2",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c1-p3",
                    "A hill can be treated as an inclined plane at 30 degrees to the "
                    "horizontal. Separately, a pail is raised from a well using a "
                    "double pulley system in which two segments of rope (two tension "
                    "forces) support the load. Ignoring friction, how does the "
                    "mechanical advantage of the hill compare to that of the double "
                    "pulley system?",
                    [
                        "They are equal.",
                        "The mechanical advantage of the hill is half that of the "
                        "double pulley system.",
                        "The mechanical advantage of the hill is double that of the "
                        "double pulley system.",
                        "It is impossible to answer without knowing how far up the "
                        "hill a person walked or how high a mass was lifted.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch34-c1-p4",
                    "Jack, who weighs 2000 N, runs up a hill inclined at 30 degrees "
                    "to the horizontal along a path 450 m long (the hypotenuse). He "
                    "takes 10 minutes to reach the halfway point and another 20 "
                    "minutes to reach the top. What was Jack's net power output "
                    "during his entire journey up the hill?",
                    ["250 W", "500 W", "2500 W", "15 kW"],
                    0,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch34-c2",
            "Kinetic energy",
            "Kinetic energy is the energy an object has due to its motion, KE = "
            "(1/2)mv^2, where m is mass and v is speed; like work it is a scalar "
            "measured in joules, and because it depends on the square of the speed, "
            "doubling the speed quadruples the kinetic energy. The work-energy "
            "theorem states that the total work done on an object equals its change "
            "in kinetic energy (W_total = delta KE). Thus positive total work "
            "speeds an object up and negative total work slows it down, and the "
            "theorem lets you find the total work without knowing the force or "
            "displacement.",
            [
                _problem(
                    "sr-mcat3135-ch34-c2-p1",
                    "Which of the following represents the kinetic energy of an "
                    "object after it has slid a distance d down a plane inclined at "
                    "an angle phi to the horizontal, assuming a constant frictional "
                    "force of magnitude F, if the object began at rest?",
                    [
                        "mgd sin(phi) - Fd",
                        "mgd cos(phi) + Fd",
                        "mgd sin(phi) + Fd",
                        "mgd cos(phi) - Fd",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch34-c2-p2",
                    "A 1000 kg car traveling 20 m/s applies its brakes and comes to "
                    "a stop over a distance of 20 m. Assuming the kinetic frictional "
                    "force was constant during braking, what is the magnitude of "
                    "that force?",
                    ["10 N", "20 N", "10^4 N", "2 x 10^4 N"],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c2-p3",
                    "At the top of a hill is a well whose water level is 15 m below "
                    "the top. A penny is dropped from the top and a splash is heard. "
                    "Sound travels at 340 m/s and g = 10 m/s^2. Approximately how "
                    "much time passes between when the penny is dropped and when the "
                    "splash is heard?",
                    ["0.04 s", "0.08 s", "1.74 s", "3.46 s"],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c2-p4",
                    "From the top of a hill, a person throws a non-uniform block of "
                    "mass m as hard as he can. Neglecting air resistance, which "
                    "statements is/are true? I. The center of mass follows a "
                    "parabolic trajectory until the block strikes the ground. "
                    "II. Assuming the block rotates, it will rotate about its "
                    "geometric center. III. The total displacement of the block from "
                    "throw to landing depends on the block's mass.",
                    ["I only", "I and II only", "I and III only", "II and III only"],
                    0,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch34-c3",
            "Potential energy",
            "Potential energy is the energy an object has by virtue of its "
            "position; gravitational potential energy changes when an object's "
            "height in a gravitational field changes, with delta PE = mg(delta h). "
            "Potential energy is measured relative to a chosen zero level, but the "
            "change in potential energy is independent of that choice, so only "
            "changes are physically meaningful. Gravity is a conservative force "
            "because the work it does depends only on the initial and final "
            "heights, not on the path taken; friction is not conservative, so there "
            "is no such thing as frictional potential energy.",
            [
                _problem(
                    "sr-mcat3135-ch34-c3-p1",
                    "Two friends start at the top of the highest platform of a "
                    "waterpark and slide down two frictionless slides to the same "
                    "pool at the bottom. One takes a nearly vertical slide, the "
                    "other a longer, winding slide with a gentler slope. Which best "
                    "describes their trips from start to finish?",
                    [
                        "Both reach the pool in the same time and at the same final "
                        "speed.",
                        "The friend on the vertical slide reaches the pool in the "
                        "same time but at a greater final speed.",
                        "The friend on the vertical slide reaches the pool in less "
                        "time but at the same final speed.",
                        "The friend on the vertical slide reaches the pool in less "
                        "time and at a greater final speed.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c3-p2",
                    "In a circus act, Brother Alexey jumps from a height of 20 m "
                    "onto a seesaw and launches Brother Boris 10 m into the air; "
                    "both brothers weigh 1000 N. Which could contribute to why Boris "
                    "is not launched to 20 m? I. Alexey lands vertically, but Boris "
                    "launches off at a 45-degree angle. II. Alexey starts with 20 kJ "
                    "of PE while Boris starts with 0 kJ. III. Not all kinetic energy "
                    "is transferred from Alexey to Boris through the seesaw.",
                    [
                        "II only",
                        "II and III only",
                        "I and III only",
                        "I, II, and III",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c3-p3",
                    "A physically fit person has a mechanical efficiency of 25% "
                    "(25% of the chemical energy converted in the body is output as "
                    "useful work, the rest wasted as heat). She has a mass of 50 kg "
                    "and climbs stairs at a constant speed, moving upward at 25 "
                    "cm/s. At what rate is she consuming energy?",
                    ["500 W", "125 W", "6.25 W", "1.56 W"],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch34-c3-p4",
                    "Using a well, a person first lowers an empty 2-liter pail 15 m "
                    "down to the water and then draws the full pail of water back up "
                    "15 m. The density of water is 1 kg/liter, the rope has "
                    "negligible mass, and g = 10 m/s^2. How much total work does the "
                    "person do in lowering the empty pail and then lifting the full "
                    "pail out of the well?",
                    [
                        "30 J",
                        "150 J",
                        "300 J",
                        "It is impossible to tell without knowing the mass of the "
                        "empty pail.",
                    ],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch34-c4",
            "Total mechanical energy",
            "An object's total mechanical energy is the sum of its kinetic and "
            "potential energy, E = KE + PE. If the only forces doing work are "
            "conservative (for example, no friction), total mechanical energy is "
            "conserved, so KE_i + PE_i = KE_f + PE_f; this is often seen as a "
            "transformation between kinetic and potential energy, as when a falling "
            "object converts gravitational PE into KE. When a nonconservative force "
            "such as friction acts, mechanical energy is not conserved, and the "
            "relation becomes KE_i + PE_i + W_friction = KE_f + PE_f, where "
            "W_friction is negative.",
            [
                _problem(
                    "sr-mcat3135-ch34-c4-p1",
                    "A basketball player jumps straight up to perform a slam dunk. "
                    "Neglecting air resistance, which statements correctly describe "
                    "the motion on the way up? I. Total mechanical energy is "
                    "conserved. II. The player's weight increases as he rises. "
                    "III. The work being done by gravity is negative.",
                    ["I only", "I and II", "I and III", "II and III"],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch34-c4-p2",
                    "An object is launched from the top of a 40 m high building with "
                    "a speed of 10 m/s at an angle of 30 degrees above the "
                    "horizontal. With what speed does it hit the ground? (g = 10 "
                    "m/s^2)",
                    ["10 m/s", "28 m/s", "29 m/s", "30 m/s"],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch34-c4-p3",
                    "In which of the following real-world cases is total mechanical "
                    "energy most nearly conserved?",
                    [
                        "A sled sliding down an icy bobsled track",
                        "A pitched baseball being hit by a bat",
                        "A car whose driver slams on the brakes to avoid a "
                        "collision",
                        "A land mine exploding",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch34-c4-p4",
                    "Jack, who weighs 2000 N (mass 200 kg), is jogging down a hill "
                    "inclined at 30 degrees. He falls and, while moving at 10 m/s, "
                    "slides 10 m down the incline before stopping at the base of the "
                    "hill (height = 0). Taking g = 10 m/s^2, how much work was done "
                    "by kinetic friction in stopping him?",
                    ["-10^4 J", "-2 x 10^4 J", "-3 x 10^4 J", "-4 x 10^4 J"],
                    1,
                ),
            ],
        ),
    ],
)


# =====================================================================
# Ch 35 - Thermodynamics
#   Concepts: Prep 2021-2022, chapter 39 (sections 39.1-39.4)
#   Problems: Workout 2024, chapter 35. NOTE: that Workout chapter has only
#   11 questions total (6 freestanding + 5 passage), so to fill 4x4 without
#   inventing anything, the last 5 problems are real 4-choice example MCQs
#   from the matching Prep chapter 39 (Examples 39-2, 39-3, 39-4, 39-6, 39-7),
#   verified against the book's solutions. Flagged so the source is explicit.
# =====================================================================

_CH35 = _leaf(
    "sr-mcat3135-ch35",
    "Thermodynamics",
    [
        _concept(
            "sr-mcat3135-ch35-c1",
            "Systems, thermal physics, and thermodynamics",
            "A system is the object or objects under examination together with the "
            "ways they interact, while the environment is everything outside it; a "
            "system is open if it can exchange matter and energy, closed if the "
            "environment cannot contribute matter, and isolated if it can "
            "contribute neither matter nor energy. Systems obey conservation of "
            "energy: energy can transform among forms, but the total energy changes "
            "only when energy is transferred into or out of the system. Thermal "
            "(internal) energy is the random molecular motion inside a system, "
            "temperature (T) is the macroscopic measure of that thermal energy per "
            "molecule, and heat (Q) is the transfer of thermal energy between a "
            "system and its environment (Q > 0 into the system). Temperature is an "
            "intensive property that does not depend on the amount of material, "
            "whereas thermal energy is extensive.",
            [
                _problem(
                    "sr-mcat3135-ch35-c1-p1",
                    "Oxygen tanks for breathing in space have a volume of 5000 cm^3 "
                    "and hold pure oxygen at a maximum pressure of 25 kPa; they are "
                    "well insulated. An astronaut connects his full tank (at maximum "
                    "pressure) to an identical tank that was vented to space and now "
                    "holds only vacuum. After the valve is opened, what is the work "
                    "W done by the oxygen in filling the second tank, and what is "
                    "the change in internal energy delta E of the oxygen once it "
                    "occupies both tanks?",
                    [
                        "W = 0 J and delta E = 0 J",
                        "W = 93 J and delta E = -93 J",
                        "W = 0 J and delta E = 125 J",
                        "W = 125 J and delta E = -125 J",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch35-c1-p2",
                    "Sections of jointed steel railroad track are installed with "
                    "expansion joints (gaps) to allow thermal expansion in hot "
                    "weather. The coefficient of linear expansion for steel is 1.0 x "
                    "10^-5 per degC. If sections of track were 25 m long when laid "
                    "at 4 degC, and the expansion joint was 10 mm wide then, what "
                    "must the track temperature be when the joint closes?",
                    ["24 degC", "40 degC", "44 degC", "84 degC"],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch35-c1-p3",
                    "A passage on thermoregulation gives the heat lost by "
                    "evaporation as Q = mL, where m is the mass of sweat evaporated "
                    "and L = 2.43 x 10^6 J/kg is the latent heat of vaporization of "
                    "sweat. Even when perspiration isn't perceptible, the body loses "
                    "roughly 0.6 kg of moisture from the skin daily. How much heat "
                    "will a person release through evaporation on a relaxing day "
                    "when they don't notice any perspiration?",
                    ["1460 kJ", "1460 kW", "4050 kJ", "4050 kW"],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch35-c1-p4",
                    "A passage on thermoregulation gives the rate of heat loss by "
                    "radiation as about k_rad x A_exp x (T_skin - T_ambient), with "
                    "k_rad = 5.6 W/(m^2 K), an adult body surface area of about 2 "
                    "m^2, and a skin temperature of about 34 degC. A person sitting "
                    "at room temperature (25 degC) with half their body covered by "
                    "clothing has a basal metabolism generating 85 W of heat. About "
                    "what percentage of that 85 W is lost through radiation?",
                    ["30%", "70%", "100%", "140%"],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch35-c2",
            "The Zeroth Law of Thermodynamics",
            "The Zeroth Law states that if one object is in thermal equilibrium "
            "with a second, and that second is in thermal equilibrium with a third, "
            "then the first and third are in thermal equilibrium with each other. "
            "Two bodies are in thermal equilibrium when heat is free to pass between "
            "them yet no net heat flows, which in practice means they are at the "
            "same temperature; this establishes temperature as a fundamental state "
            "variable (others include pressure, volume, moles, and entropy). Bodies "
            "reach thermal equilibrium through heat transfer, which occurs by three "
            "mechanisms: conduction (direct molecular collisions), convection (bulk "
            "motion of a fluid), and radiation (energy carried by electromagnetic "
            "waves).",
            [
                _problem(
                    "sr-mcat3135-ch35-c2-p1",
                    "On a hot, sunny, windless day, you lie on a towel on the sandy "
                    "beach. After a few minutes you feel quite a bit warmer. What "
                    "best explains this?",
                    [
                        "The hot sand transferred heat into you by conduction and "
                        "the Sun's rays transferred heat into you by radiation.",
                        "The hot sand transferred heat into you by conduction and "
                        "the hot air transferred heat into you by convection.",
                        "The hot sand transferred heat into you by convection and "
                        "the Sun's rays transferred heat into you by radiation.",
                        "All of your accumulated heat came from conduction by the "
                        "surrounding air.",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch35-c2-p2",
                    "In a thermoregulation passage, heat loss by radiation and by "
                    "convection both depend on the exposed skin area A_exp, so "
                    "reducing A_exp reduces both; heat loss by conduction depends on "
                    "the conductivity and thickness of whatever covers the skin "
                    "(still air is less conductive than wool or cotton), so covering "
                    "skin has an ambiguous effect on conduction. Given this, why do "
                    "people tend to cover most of their bodies with clothing in "
                    "winter?",
                    [
                        "To increase the heat created by the body",
                        "To increase heat loss through conduction",
                        "To reduce heat loss through conduction",
                        "To reduce heat loss through radiation and convection",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch35-c2-p3",
                    "A thermoregulation passage states that the body loses heat by "
                    "conduction about 25 times faster in water than in air, that "
                    "heat loss by conduction and radiation do not depend on motion, "
                    "and that heat loss by convection increases with the speed of "
                    "the fluid flowing across the skin. Why are people who fall into "
                    "cold water told to remain as still as possible while awaiting "
                    "assistance?",
                    [
                        "Remaining still reduces the heat generated in the body "
                        "through metabolism.",
                        "Remaining still reduces the rate of heat loss via "
                        "conduction.",
                        "Remaining still reduces the rate of heat loss via "
                        "convection.",
                        "Remaining still reduces the rate of heat loss via "
                        "radiation.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch35-c2-p4",
                    "During circulation, relatively warm blood moves from the heart "
                    "to the extremities, where it cools slightly before returning to "
                    "the heart. What best describes this process?",
                    [
                        "A free convection process, as warm blood rises to the head "
                        "while colder blood sinks to the feet.",
                        "A free convection process, as warm blood expands in the "
                        "heart and is pushed out via the arteries, while cooler "
                        "blood at the extremities condenses and sinks back via the "
                        "veins.",
                        "A forced convection process, in which the heart's pumping "
                        "forces metabolically heated blood out to the extremities, "
                        "which in turn forces cooler blood back toward the heart.",
                        "A forced convection process, in which the heart's pumping "
                        "compresses and thereby heats the blood, driving its motion "
                        "to the extremities.",
                    ],
                    2,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch35-c3",
            "The First Law of Thermodynamics",
            "The First Law is a statement of conservation of energy: the total "
            "energy of the universe is constant, so energy may change form but "
            "cannot be created or destroyed. Because energy transferred to or from "
            "a system takes the form of heat (Q) or work (W), the law is written "
            "delta E = Q - W. In this physics sign convention Q is positive when "
            "heat flows into the system and W is positive when the system does work "
            "on the environment (chemistry texts use the opposite work sign, giving "
            "delta E = Q + W). The internal energy of an ideal gas is proportional "
            "to its absolute temperature, and the work done by an expanding gas is "
            "W = P(delta V), so in an isochoric (constant-volume) process no work "
            "is done and delta E = Q.",
            [
                _problem(
                    "sr-mcat3135-ch35-c3-p1",
                    "A mole of ideal gas undergoes an isochoric process. Which of "
                    "the following quantities remains constant?",
                    [
                        "The internal energy, E",
                        "The heat, Q",
                        "T/P",
                        "PV",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch35-c3-p2",
                    "A perfectly insulated, frictionless piston of ideal gas starts "
                    "at 2000 Pa and 0.1 m^3, then runs a reversible cycle "
                    "1->2->3->4->1 back to its start. Along 1->2 the gas expands "
                    "from 0.1 to 0.5 m^3 while its pressure changes linearly between "
                    "2000 Pa and 1000 Pa. Paths 2->3 and 4->1 are isochoric (no "
                    "work). Along 3->4 the gas is compressed from 0.5 back to 0.1 "
                    "m^3 while its pressure changes linearly between 6000 Pa and "
                    "4000 Pa. What is the net work done by the gas over the cycle?",
                    ["-1400 J", "600 J", "1400 J", "2000 J"],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch35-c3-p3",
                    "For a perfectly insulated system, what are the values of delta "
                    "E_internal and Q if W = +100 J?",
                    [
                        "delta E_internal = -100 J and Q = 0",
                        "delta E_internal = 0 and Q = -100 J",
                        "delta E_internal = +100 J and Q = 0",
                        "delta E_internal = 0 and Q = +100 J",
                    ],
                    0,
                ),
                _problem(
                    "sr-mcat3135-ch35-c3-p4",
                    "Suppose you want to raise the temperature of an ideal gas while "
                    "adding the lowest possible amount of heat and doing no work on "
                    "the gas. Which process should you use?",
                    ["Isobaric", "Isochoric", "Isothermal", "Adiabatic"],
                    1,
                ),
            ],
        ),
        _concept(
            "sr-mcat3135-ch35-c4",
            "The Second Law of Thermodynamics",
            "Entropy is a measure of the disorder of a system, so solids have less "
            "entropy than liquids, which have less than gases. The Second Law "
            "states that the entropy of an isolated system either stays the same or "
            "increases during any thermodynamic process; the entropy of a closed "
            "system can decrease only if the surroundings' entropy increases by a "
            "greater amount. A process is reversible if entropy is unchanged and "
            "irreversible if entropy increases, and because of friction and other "
            "losses all real macroscopic processes are irreversible. As a result, "
            "it is impossible to convert all input heat into work over a cycle "
            "(some heat must always be exhausted), and heat cannot flow from a "
            "colder body to a hotter one without an input of work.",
            [
                _problem(
                    "sr-mcat3135-ch35-c4-p1",
                    "To cool your coffee, you drop an ice cube into your thermos "
                    "(which prevents heat transfer in or out of the contents), close "
                    "the lid tightly, and wait for the cube to melt. Which best "
                    "describes the situation during the waiting period?",
                    [
                        "The total energy of the system (coffee + ice) decreased and "
                        "the entropy stayed constant.",
                        "The total energy of the system and the entropy both stayed "
                        "constant.",
                        "The total energy of the system decreased and the entropy "
                        "increased.",
                        "The total energy of the system stayed constant and the "
                        "entropy increased.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch35-c4-p2",
                    "In a thermoregulation passage, skin temperature is about 34 "
                    "degC, and heat loss by radiation, convection, and conduction "
                    "are each proportional to (T_skin - T_ambient), carrying heat "
                    "out only when the surroundings are cooler than the skin. "
                    "Evaporation of sweat is the one mechanism that does not depend "
                    "on that difference, and the body perspires when core "
                    "temperature exceeds about 36.85 degC. If the ambient "
                    "temperature rises above skin temperature, what happens to a "
                    "person?",
                    [
                        "Since heat can't transfer from lower to higher temperature, "
                        "body temperature rises, culminating in heat stroke.",
                        "The core temperature rises, so the body radiates more heat "
                        "to the environment, cooling the core back within range.",
                        "Perspiration increases, as evaporation becomes the only "
                        "heat-loss mechanism available to keep the core within "
                        "range.",
                        "The body reduces the heat it generates through metabolism "
                        "to keep the core within range.",
                    ],
                    2,
                ),
                _problem(
                    "sr-mcat3135-ch35-c4-p3",
                    "An ideal gas is held under pressure in an isolated container "
                    "behind a thin membrane separating it from vacuum, like an "
                    "inflated balloon in a large evacuated room. The membrane is "
                    "suddenly ruptured, like popping the balloon. What happens next?",
                    [
                        "Nothing: the membrane ruptures, but the gas is unaffected.",
                        "The pressure and temperature both decrease rapidly.",
                        "The temperature decreases rapidly, but the pressure stays "
                        "constant.",
                        "The pressure decreases rapidly, but the temperature remains "
                        "constant, since no heat is exchanged and no work is done.",
                    ],
                    3,
                ),
                _problem(
                    "sr-mcat3135-ch35-c4-p4",
                    "A patient recovering from surgery is confined to a bed for the "
                    "first couple of days, with minimal movement. During that time, "
                    "incisions heal and organ function returns to normal. What best "
                    "explains this?",
                    [
                        "The person's entropy decreases, but the entropy of the "
                        "surroundings increases more.",
                        "Because the person does no work, she is an isolated system, "
                        "so her entropy increases.",
                        "The person absorbs more heat from the environment than she "
                        "emits, increasing her internal energy.",
                        "There is no change to the person's entropy because healing "
                        "is a closed thermodynamic cycle.",
                    ],
                    0,
                ),
            ],
        ),
    ],
)


HIERARCHY: dict[str, Any] = {
    "deckId": "new",
    "root": _branch(
        "sr-mcat3135-root",
        DECK_NAME,
        [
            _branch(
                "sr-mcat3135-orgo",
                "Organic Chemistry",
                [_CH31, _CH32],
            ),
            _branch(
                "sr-mcat3135-physics",
                "Physics",
                [_CH33, _CH34, _CH35],
            ),
        ],
    ),
}
