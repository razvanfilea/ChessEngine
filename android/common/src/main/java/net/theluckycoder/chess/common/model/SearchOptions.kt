package net.theluckycoder.chess.common.model

import androidx.annotation.Keep
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
@Keep
data class SearchOptions(
    val searchDepth: Int,
    val threadCount: Int,
    val searchTime: Duration,
    val hashSize: Int,
) {
    companion object {
        val DEFAULT = SearchOptions(
            searchDepth = 8,
            threadCount = 1,
            searchTime = 10.seconds,
            hashSize = 64,
        )
    }
}
