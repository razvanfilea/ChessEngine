package cloud.razvan.chess.common.model

import androidx.annotation.Keep

@Keep
class BoardState(
    val gameState: Int,
    val evalScore: Int,
    val searchTimeMs: Long,
    val uciInfo: String,
    val isWhiteTurn: Boolean,
    val currentMoveIndex: Int,
    val pieces: IntArray,
    val movesHistory: IntArray,
    val movesSan: Array<String>,
    val legalMoves: IntArray,
)
