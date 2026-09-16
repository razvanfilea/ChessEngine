package net.theluckycoder.chess.common.viewmodel

import android.app.Application
import androidx.compose.runtime.mutableStateOf
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.theluckycoder.chess.common.SaveManager
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.BoardState
import net.theluckycoder.chess.common.model.DebugStats
import net.theluckycoder.chess.common.model.GameState
import net.theluckycoder.chess.common.model.IndexedPiece
import net.theluckycoder.chess.common.model.Move
import net.theluckycoder.chess.common.model.Tile

class HomeViewModel(application: Application) : AndroidViewModel(application) {

    private var initialized = false
    private var searchJob: Job? = null
    val dataStore = SettingsDataStore.get(application)

    /*
     * Chess Game Data
     */
    private val isEngineBusyFlow = MutableStateFlow(false)
    private val playerPlayingWhiteFlow = MutableStateFlow(true)
    private val isWhiteTurnFlow = MutableStateFlow(true)
    private val tilesFlow = MutableStateFlow(getEmptyTiles())
    private val piecesFlow = MutableStateFlow(emptyList<IndexedPiece>())
    private val gameStateFlow = MutableStateFlow(GameState.NONE)
    private val movesHistoryFlow = MutableStateFlow(emptyList<Move>())
    private val currentMoveIndexFlow = MutableStateFlow(0)
    private val debugStatsFlow = MutableStateFlow(DebugStats())

    val playerPlayingWhite: StateFlow<Boolean> = playerPlayingWhiteFlow
    val isEngineBusy: StateFlow<Boolean> = isEngineBusyFlow
    val tiles: StateFlow<List<Tile>> = tilesFlow
    val pieces: StateFlow<List<IndexedPiece>> = piecesFlow
    val gameState: StateFlow<GameState> = gameStateFlow
    val movesHistory: StateFlow<List<Move>> = movesHistoryFlow
    val currentMoveIndex: StateFlow<Int> = currentMoveIndexFlow
    val debugStats: StateFlow<DebugStats> = debugStatsFlow

    /*
     * UI
     */
    val showNewGameDialog = mutableStateOf(false)
    val showShareDialog = mutableStateOf(false)
    val showImportDialog = mutableStateOf(false)

    init {
        resetBoard()

        viewModelScope.launch(Dispatchers.IO) {
            launch {
                dataStore.showBasicDebug().distinctUntilChanged().collectLatest { enabled ->
                    if (enabled) {
                        debugStatsFlow.value = DebugStats.fromBoardState(Native.getBoardState())
                    }
                }
            }

            launch {
                movesHistoryFlow.collectLatest {
                    ensureActive()
                    val state = gameState.value
                    val currentIndex = currentMoveIndexFlow.value
                    if (it.isNotEmpty() && currentIndex >= 0 && state != GameState.WINNER_BLACK && state != GameState.WINNER_WHITE && state != GameState.DRAW) {
                        val activeMoves = it.take(currentIndex + 1)
                        SaveManager.saveToFileAsync(
                            getApplication(),
                            Native.getStartFen(),
                            playerPlayingWhite.value,
                            activeMoves
                        )
                    }
                }
            }
        }
    }

    fun resetBoard(playerWhite: Boolean = true) {
        stopSearch()

        if (initialized) {
            playerPlayingWhiteFlow.value = playerWhite
            val state = Native.initBoard(playerWhite)
            onBoardUpdated(state)
        } else {
            // First time it is called, load the last game
            val saved = SaveManager.loadFromFile(getApplication())
            val state = if (saved != null) {
                playerPlayingWhiteFlow.value = saved.playerWhite
                saved.state
            } else {
                playerPlayingWhiteFlow.value = true
                Native.initBoard(true)
            }
            initialized = true
            onBoardUpdated(state)
        }
    }

    fun loadFen(playerWhite: Boolean, fen: String): Boolean {
        stopSearch()
        val state = Native.loadFenMoves(fen, isPlayerWhite = playerWhite) ?: return false
        playerPlayingWhiteFlow.value = playerWhite
        onBoardUpdated(state)
        return true
    }

    fun updateDifficulty(level: Int) = viewModelScope.launch(Dispatchers.IO) {
        dataStore.setDifficultyLevel(level)
    }

    private var selectedSquare: Int? = null

    fun getCurrentFen(): String = Native.getCurrentFen()
    fun getStartFen(): String = Native.getStartFen()

    fun showPossibleMoves(square: Int) {
        if (selectedSquare == square) {
            selectedSquare = null
            updateTiles(emptyList())
            return
        }

        val rawMoves = Native.getPossibleMoves(square.toByte())
        if (rawMoves.isEmpty()) {
            selectedSquare = null
            updateTiles(emptyList())
            return
        }

        selectedSquare = square
        val moves = rawMoves.map { Move(it) }
        updateTiles(moves)
    }

    private fun updateTiles(possibleMoves: List<Move>) {
        val currentMove = movesHistoryFlow.value.getOrNull(currentMoveIndexFlow.value)
        val movesByDest = possibleMoves.groupBy { it.to.toInt() }

        tilesFlow.value = List(64) { sq ->
            val destMoves = movesByDest[sq]
            val state = when {
                destMoves != null -> Tile.State.PossibleMove(destMoves)
                sq == selectedSquare -> Tile.State.Selected
                currentMove != null && (sq.toByte() == currentMove.from || sq.toByte() == currentMove.to) -> Tile.State.Moved
                else -> Tile.State.None
            }
            Tile(sq, state)
        }
    }

    fun makeMove(move: Move) {
        val state = Native.makeMove(move)
        onBoardUpdated(state)
    }

    fun makeMove(moveContent: Int) {
        val state = Native.makeMove(moveContent)
        onBoardUpdated(state)
    }

    fun undo() {
        stopSearch()
        val state = Native.undo() ?: return
        onBoardUpdated(state)
    }

    fun redo() {
        stopSearch()
        val state = Native.redo() ?: return
        onBoardUpdated(state)
    }

    private fun onBoardUpdated(state: BoardState) {
        selectedSquare = null
        isWhiteTurnFlow.value = state.isWhiteTurn
        val newGameState = GameState.getState(state.gameState)
        gameStateFlow.value = newGameState

        val movesHistoryList = state.movesHistory.map { Move(it) }
        movesHistoryFlow.value = movesHistoryList
        currentMoveIndexFlow.value = state.currentMoveIndex
        debugStatsFlow.value = DebugStats.fromBoardState(state)

        updateTiles(emptyList())

        piecesFlow.value = state.pieces.map { IndexedPiece(it) }
        triggerEngineMove()
    }

    fun triggerEngineMove(force: Boolean = false) {
        if (!initialized) return
        val isPlayersTurn = isWhiteTurnFlow.value == playerPlayingWhite.value
        if (!force && isPlayersTurn) return
        if (currentMoveIndex.value != movesHistory.value.lastIndex) return

        val state = gameState.value
        if (state == GameState.WINNER_BLACK || state == GameState.WINNER_WHITE || state == GameState.DRAW) {
            return
        }

        searchJob?.cancel()
        searchJob = viewModelScope.launch(Dispatchers.Default) {
            isEngineBusyFlow.value = true
            val settings = dataStore.getEngineSettings().first()
            val bestMove = Native.search(
                settings.searchDepth,
                settings.searchTime.inWholeMilliseconds,
                settings.hashSize,
                settings.threadCount
            )
            isEngineBusyFlow.value = false

            if (isActive && bestMove != 0) {
                withContext(Dispatchers.Main) {
                    makeMove(bestMove)
                }
            }
        }
    }

    fun stopSearch() {
        searchJob?.cancel()
        Native.stopSearch()
        isEngineBusyFlow.value = false
    }

    override fun onCleared() {
        super.onCleared()
        searchJob?.cancel()
        Native.stopSearch()
    }

    private companion object {
        private val emptyTiles = Array(64) { Tile(it, Tile.State.None) }
        fun getEmptyTiles() = emptyTiles.toList()
    }
}
