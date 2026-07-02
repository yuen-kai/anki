# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html


"""
The V3/2021 scheduler.

https://faqs.ankiweb.net/the-2021-scheduler.html

It uses the same DB schema as the V2 scheduler, and 'schedVer' remains
as '2' internally.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Any, Literal

from anki import frontend_pb2, scheduler_pb2
from anki._legacy import deprecated
from anki.cards import Card, CardId
from anki.collection import OpChanges
from anki.consts import *
from anki.decks import DeckId
from anki.errors import DBError
from anki.scheduler.legacy import SchedulerBaseWithLegacy
from anki.types import assert_exhaustive
from anki.utils import int_time

QueuedCards = scheduler_pb2.QueuedCards
SchedulingState = scheduler_pb2.SchedulingState
SchedulingStates = scheduler_pb2.SchedulingStates
SchedulingContext = scheduler_pb2.SchedulingContext
SchedulingStatesWithContext = frontend_pb2.SchedulingStatesWithContext
SetSchedulingStatesRequest = frontend_pb2.SetSchedulingStatesRequest
CardAnswer = scheduler_pb2.CardAnswer


class Scheduler(SchedulerBaseWithLegacy):
    version = 3

    # don't rely on this, it will likely be removed in the future
    reps = 0

    # Fetching the next card
    ##########################################################################

    def get_queued_cards(
        self,
        *,
        fetch_limit: int = 1,
        intraday_learning_only: bool = False,
    ) -> QueuedCards:
        "Returns zero or more pending cards, and the remaining counts. Idempotent."
        return self.col._backend.get_queued_cards(
            fetch_limit=fetch_limit, intraday_learning_only=intraday_learning_only
        )

    def get_topic_grouped_queue(
        self,
        *,
        deck_id: DeckId,
        fetch_limit: int = 0,
    ) -> QueuedCards:
        """Speedrun "Learn" mode: a deck's due review cards regrouped into
        topic-contiguous blocks, ordered by weakness × exam weight.

        Read-only: grading still flows through answer_card, so FSRS intervals and
        undo are unchanged. fetch_limit caps the returned cards (0 = no limit);
        review_count always reports the full block total. Idempotent."""
        return self.col._backend.get_topic_grouped_queue(
            deck_id=deck_id, fetch_limit=fetch_limit
        )

    def get_memory_score(self, *, deck_id: DeckId) -> scheduler_pb2.MemoryScore:
        """Speedrun honest Memory score for a deck: aggregated FSRS
        retrievability over its in-scope taxonomy cards, returned as the full
        evidence envelope (estimate, range, coverage, confidence, reasons).

        Read-only: reads existing FSRS/revlog state only. When the give-up rule
        fires (too few graded reviews or too little topic coverage) the score
        abstains — estimate/range are 0 and abstain_reason says what's missing."""
        return self.col._backend.get_memory_score(deck_id=deck_id)

    def get_performance_score(self, *, deck_id: DeckId) -> scheduler_pb2.ScoreEnvelope:
        """Speedrun honest Performance score for a deck: weighted accuracy over
        graded reviews of SpeedrunApplication (exam-style) cards, as the full
        evidence envelope. Distinct from Memory (recall of a taught fact); this
        measures getting application questions right. Read-only; abstains under
        the give-up rule (too few application attempts or too little coverage)."""
        return self.col._backend.get_performance_score(deck_id=deck_id)

    def get_readiness_score(self, *, deck_id: DeckId) -> scheduler_pb2.ScoreEnvelope:
        """Speedrun honest Readiness score for a deck: the Performance score
        projected onto the 472-528 MCAT scale with a coverage-widened range, as
        the full evidence envelope (format "points"). Abstains whenever
        Performance abstains; a projection from practice, not calibrated to real
        exam outcomes yet. Read-only."""
        return self.col._backend.get_readiness_score(deck_id=deck_id)

    def get_speedrun_card_mode(self, *, card_id: CardId) -> str:
        """Speedrun mastery progression: the card's active mode, resolved from
        its topic's state and note type. One of "concept_learn",
        "concept_practice", "application_scaffolded", "application_unscaffolded",
        or "none" (a non-Speedrun card, or an application card whose topic is
        below "hierarchy" and is suppressed). The reviewer injects this as
        window.speedrunCardMode before render."""
        return self.col._backend.get_speedrun_card_mode(card_id)

    def get_speedrun_card_context(
        self, *, card_id: CardId
    ) -> scheduler_pb2.SpeedrunCardContext:
        """Speedrun mastery progression: the card's render context — its active
        mode (as get_speedrun_card_mode) plus its taxonomy hierarchy path as
        display labels (foundation -> leaf, e.g. "Biomolecules", "Enzymes",
        "Inhibition"), empty for a card with no taxonomy topic. The reviewer
        injects these as window.speedrunCardMode and window.speedrunTopicPath
        before render, so the templates can show the breadcrumb."""
        return self.col._backend.get_speedrun_card_context(card_id)

    def speedrun_record_answer(
        self, *, card_id: CardId, rating: CardAnswer.Rating.V
    ) -> str:
        """Record an answer against the card's topic and return the topic's new
        mastery state ("learning"/"practicing"/"hierarchy"/"mastering"). Again
        demotes one state (never below "learning"); any other rating advances one
        state once the topic's active-mode cards clear the mastery signal.

        Writes only the per-topic state map in the collection config — no
        scheduling change, so the card's FSRS state and undo are untouched. Call
        it alongside answer_card, not instead of it."""
        return self.col._backend.speedrun_record_answer(card_id=card_id, rating=rating)

    def get_speedrun_progress(
        self, *, deck_id: DeckId
    ) -> Sequence[scheduler_pb2.SpeedrunProgress.TopicProgress]:
        """Per-topic mastery state ((topic_id, state) pairs) for the deck's
        in-scope topics, for the dashboard's per-area progress. Read-only."""
        return self.col._backend.get_speedrun_progress(deck_id)

    def describe_next_states(self, next_states: SchedulingStates) -> Sequence[str]:
        "Labels for each of the answer buttons."
        return self.col._backend.describe_next_states(next_states)

    # Answering a card
    ##########################################################################

    def build_answer(
        self,
        *,
        card: Card,
        states: SchedulingStates,
        rating: CardAnswer.Rating.V,
        from_queue: bool = True,
    ) -> CardAnswer:
        """Build input for answer_card().

        ``from_queue`` should be False when the card was not served from the live
        study queue (e.g. the Speedrun Learn topic-grouped ordering), so the
        backend skips popping it from the queue and doesn't fail with "not at top
        of queue".
        """
        if rating == CardAnswer.AGAIN:
            new_state = states.again
        elif rating == CardAnswer.HARD:
            new_state = states.hard
        elif rating == CardAnswer.GOOD:
            new_state = states.good
        elif rating == CardAnswer.EASY:
            new_state = states.easy
        else:
            raise Exception("invalid rating")

        return CardAnswer(
            card_id=card.id,
            current_state=states.current,
            new_state=new_state,
            rating=rating,
            answered_at_millis=int_time(1000),
            milliseconds_taken=card.time_taken(capped=False),
            from_queue=from_queue,
        )

    def answer_card(self, input: CardAnswer) -> OpChanges:
        "Update card to provided state, and remove it from queue."
        self.reps += 1
        op_bytes = self.col._backend.answer_card_raw(input.SerializeToString())
        return OpChanges.FromString(op_bytes)

    def state_is_leech(self, new_state: SchedulingState) -> bool:
        "True if new state marks the card as a leech."
        return self.col._backend.state_is_leech(new_state)

    # Fetching the next card (legacy API)
    ##########################################################################

    @deprecated(info="no longer required")
    def reset(self) -> None:
        # backend automatically resets queues as operations are performed
        pass

    def getCard(self) -> Card | None:
        """Fetch the next card from the queue. None if finished."""
        try:
            queued_card = self.get_queued_cards().cards[0]
        except IndexError:
            return None

        card = Card(self.col)
        card._load_from_backend_card(queued_card.card)
        card.start_timer()
        return card

    def _is_finished(self) -> bool:
        "Don't use this, it is a stop-gap until this code is refactored."
        return not self.get_queued_cards().cards

    def counts(self, card: Card | None = None) -> tuple[int, int, int]:
        info = self.get_queued_cards()
        return (info.new_count, info.learning_count, info.review_count)

    @property
    def newCount(self) -> int:
        return self.counts()[0]

    @property
    def lrnCount(self) -> int:
        return self.counts()[1]

    @property
    def reviewCount(self) -> int:
        return self.counts()[2]

    def nextIvlStr(self, card: Card, ease: int, short: bool = False) -> str:
        "Return the next interval for CARD as a string."
        states = self.col._backend.get_scheduling_states(card.id)
        return self.col._backend.describe_next_states(states)[ease - 1]

    # Answering a card (legacy API)
    ##########################################################################

    def answerCard(self, card: Card, ease: Literal[1, 2, 3, 4]) -> OpChanges:
        if ease == BUTTON_ONE:
            rating = CardAnswer.AGAIN
        elif ease == BUTTON_TWO:
            rating = CardAnswer.HARD
        elif ease == BUTTON_THREE:
            rating = CardAnswer.GOOD
        elif ease == BUTTON_FOUR:
            rating = CardAnswer.EASY
        else:
            raise Exception("invalid ease")

        states = self.col._backend.get_scheduling_states(card.id)
        changes = self.answer_card(
            self.build_answer(card=card, states=states, rating=rating)
        )

        # tests assume card will be mutated, so we need to reload it
        card.load()

        return changes

    # Next times (legacy API)
    ##########################################################################
    # fixme: move these into tests_schedv2 in the future

    def _interval_for_state(self, state: scheduler_pb2.SchedulingState) -> int:
        kind = state.WhichOneof("kind")
        if kind == "normal":
            return self._interval_for_normal_state(state.normal)
        elif kind == "filtered":
            return self._interval_for_filtered_state(state.filtered)
        else:
            assert_exhaustive(kind)
            return 0

    def _interval_for_normal_state(
        self, normal: scheduler_pb2.SchedulingState.Normal
    ) -> int:
        kind = normal.WhichOneof("kind")
        if kind == "new":
            return 0
        elif kind == "review":
            return normal.review.scheduled_days * 86400
        elif kind == "learning":
            return normal.learning.scheduled_secs
        elif kind == "relearning":
            return normal.relearning.learning.scheduled_secs
        else:
            assert_exhaustive(kind)
            return 0

    def _interval_for_filtered_state(
        self, filtered: scheduler_pb2.SchedulingState.Filtered
    ) -> int:
        kind = filtered.WhichOneof("kind")
        if kind == "preview":
            return filtered.preview.scheduled_secs
        elif kind == "rescheduling":
            return self._interval_for_normal_state(filtered.rescheduling.original_state)
        else:
            assert_exhaustive(kind)
            return 0

    def nextIvl(self, card: Card, ease: int) -> Any:
        "Don't use this - it is only required by tests, and will be moved in the future."
        states = self.col._backend.get_scheduling_states(card.id)
        if ease == BUTTON_ONE:
            new_state = states.again
        elif ease == BUTTON_TWO:
            new_state = states.hard
        elif ease == BUTTON_THREE:
            new_state = states.good
        elif ease == BUTTON_FOUR:
            new_state = states.easy
        else:
            raise Exception("invalid ease")

        return self._interval_for_state(new_state)

    # Other legacy
    ###################

    # called by col.decks.active(), which add-ons are using
    @property
    def active_decks(self) -> list[DeckId]:
        try:
            return self.col.db.list("select id from active_decks")
        except DBError:
            return []
