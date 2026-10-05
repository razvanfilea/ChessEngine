package cloud.razvan.chess.common.model

// Order must match `GameState` in chess_android/src/rules.rs
enum class GameState {
    NONE,
    WINNER_WHITE,
    WINNER_BLACK,
    DRAW,
    WHITE_IN_CHECK,
    BLACK_IN_CHECK;

    val isGameOver: Boolean
        get() = this == WINNER_WHITE || this == WINNER_BLACK || this == DRAW

    companion object {
        fun getState(gameState: Int) = entries.getOrElse(gameState) { NONE }
    }
}
