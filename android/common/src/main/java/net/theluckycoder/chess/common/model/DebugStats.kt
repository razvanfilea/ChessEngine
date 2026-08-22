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
        fun get() = DebugStats(
            getNativeSearchTime().milliseconds,
            getNativeBoardEvaluation(),
            getNativeAdvancedStats(),
        )

        @JvmStatic
        private external fun getNativeSearchTime(): Long

        @JvmStatic
        private external fun getNativeBoardEvaluation(): Int

        @JvmStatic
        private external fun getNativeAdvancedStats(): String

        @JvmStatic
        external fun enable(enabled: Boolean)
    }
}
