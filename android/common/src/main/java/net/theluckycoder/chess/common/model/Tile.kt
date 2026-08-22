package net.theluckycoder.chess.common.model

data class Tile(
    val square: Int,
    val state: State
) {

    sealed class State {
        object None : State()
        class PossibleMove(val moves: List<Move>) : State()
        object Selected : State()
        object Moved : State()
    }
}
