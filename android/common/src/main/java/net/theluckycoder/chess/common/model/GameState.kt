package net.theluckycoder.chess.common.model

import androidx.annotation.Keep

@Keep
enum class GameState {
    NONE,
    WINNER_WHITE,
    WINNER_BLACK,
    DRAW,
    WHITE_IN_CHECK,
    BLACK_IN_CHECK,
    INVALID;

    companion object {
        fun getState(gameState: Int) = when (gameState) {
            1 -> WINNER_WHITE
            2 -> WINNER_BLACK
            3 -> DRAW
            4 -> WHITE_IN_CHECK
            5 -> BLACK_IN_CHECK
            10 -> INVALID
            else -> NONE
        }
    }
}
