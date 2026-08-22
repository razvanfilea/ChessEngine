package net.theluckycoder.chess.common.model

import androidx.annotation.Keep
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.ExperimentalTime

@OptIn(ExperimentalTime::class)
@Keep
data class SearchOptions(
    val searchDepth: Int,
    val threadCount: Int,
    val searchTime: Duration,
    val hashSize: Int,
    val quietSearch: Boolean,
) {
    @Keep
    constructor(
        searchDepth: Int,
        quietSearch: Boolean,
        threadCount: Int,
        searchTime: Long,
        hashSize: Int,
    ) : this(searchDepth, threadCount, searchTime.milliseconds, hashSize, quietSearch)

    companion object {

        @JvmStatic
        external fun getNativeSearchOptions(): SearchOptions

        fun setNativeSearchOptions(options: SearchOptions) = setNativeSearchOptions(
            options.searchDepth,
            options.quietSearch,
            options.threadCount,
            options.hashSize,
            options.searchTime.inWholeMilliseconds,
        )

        @JvmStatic
        private external fun setNativeSearchOptions(
            searchDepth: Int, quietSearch: Boolean,
            threadCount: Int, hashSizeMb: Int,
            searchTime: Long,
        )
    }
}
