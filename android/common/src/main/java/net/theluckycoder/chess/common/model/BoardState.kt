package net.theluckycoder.chess.common.model

import androidx.annotation.Keep

@Keep
data class BoardState(
    val gameState: Int,
    val evalScore: Int,
    val searchTimeMs: Long,
    val uciInfo: String,
    val isWhiteTurn: Boolean,
    val currentMoveIndex: Int,
    val pieces: IntArray,
    val movesHistory: IntArray,
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as BoardState

        if (gameState != other.gameState) return false
        if (evalScore != other.evalScore) return false
        if (searchTimeMs != other.searchTimeMs) return false
        if (uciInfo != other.uciInfo) return false
        if (isWhiteTurn != other.isWhiteTurn) return false
        if (currentMoveIndex != other.currentMoveIndex) return false
        if (!pieces.contentEquals(other.pieces)) return false
        if (!movesHistory.contentEquals(other.movesHistory)) return false

        return true
    }

    override fun hashCode(): Int {
        var result = gameState
        result = 31 * result + evalScore
        result = 31 * result + searchTimeMs.hashCode()
        result = 31 * result + uciInfo.hashCode()
        result = 31 * result + isWhiteTurn.hashCode()
        result = 31 * result + currentMoveIndex
        result = 31 * result + pieces.contentHashCode()
        result = 31 * result + movesHistory.contentHashCode()
        return result
    }
}
