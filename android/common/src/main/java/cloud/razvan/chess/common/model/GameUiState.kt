package cloud.razvan.chess.common.model

import kotlin.time.Duration

data class GameUiState(
    val isPlayerWhite: Boolean = true,
    val isWhiteTurn: Boolean = true,
    val pieces: List<IndexedPiece> = emptyList(),
    val selectedSquare: Int? = null,
    val legalMoves: List<Move> = emptyList(),
    val gameState: GameState = GameState.NONE,
    val movesHistory: List<Move> = emptyList(),
    val currentMoveIndex: Int = -1,
    val isEngineBusy: Boolean = false,
    val searchTime: Duration = Duration.ZERO,
    val evaluation: Int = 0,
    val uciInfo: String = "",
) {
    val isPlayersTurn: Boolean get() = isWhiteTurn == isPlayerWhite
    val possibleMoves: List<Move> get() = legalMoves.filter { it.from.toInt() == selectedSquare }
    val lastMove: Move? get() = movesHistory.getOrNull(currentMoveIndex)
    val canUndo: Boolean get() = currentMoveIndex >= 1 || (currentMoveIndex == 0 && !isPlayersTurn)
    val canRedo: Boolean get() = currentMoveIndex != movesHistory.lastIndex
}
