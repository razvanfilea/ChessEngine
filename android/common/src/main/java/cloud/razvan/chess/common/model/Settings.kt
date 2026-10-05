package cloud.razvan.chess.common.model

data class Settings(
    val showCoordinates: Boolean = true,
    val showMovesHistory: Boolean = true,
    val showCapturedPieces: Boolean = true,
    val showPossibleMoves: Boolean = true,

    val difficultyLevel: Int = 1,
    val searchTimeSeconds: Int = 30,
    val threads: Int = 1,
    val hashSizeMb: Int = 64,

    val showDebugBasic: Boolean = false,
    val showDebugAdvanced: Boolean = false,
)
