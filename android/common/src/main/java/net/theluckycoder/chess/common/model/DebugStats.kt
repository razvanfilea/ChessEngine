package net.theluckycoder.chess.common.model

import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
data class DebugStats(
    val searchTimeNeeded: Duration,
    val boardEvaluation: Int,
    val advancedStats: String,
) {

    constructor() : this(Duration.ZERO, 0, "")

    companion object {
        fun fromBoardState(state: BoardState) = DebugStats(
            searchTimeNeeded = state.searchTimeMs.milliseconds,
            boardEvaluation = state.evalScore,
            advancedStats = state.uciInfo,
        )
    }
}
