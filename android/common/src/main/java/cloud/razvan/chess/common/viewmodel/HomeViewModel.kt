package cloud.razvan.chess.common.viewmodel

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import cloud.razvan.chess.common.SaveManager
import cloud.razvan.chess.common.SettingsDataStore
import cloud.razvan.chess.common.Native
import cloud.razvan.chess.common.model.BoardState
import cloud.razvan.chess.common.model.GameState
import cloud.razvan.chess.common.model.GameUiState
import cloud.razvan.chess.common.model.IndexedPiece
import cloud.razvan.chess.common.model.Move
import cloud.razvan.chess.common.model.Settings
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

class HomeViewModel(application: Application) : AndroidViewModel(application) {

    private val dataStore = SettingsDataStore.get(application)
    private var searchJob: Job? = null

    private val stateFlow = MutableStateFlow(GameUiState())
    private val engineMovesFlow = MutableSharedFlow<Move>(extraBufferCapacity = 1)
    private val saveFlow = MutableStateFlow<Pair<Boolean, String>?>(null)

    val state: StateFlow<GameUiState> = stateFlow
    val engineMoves: SharedFlow<Move> = engineMovesFlow
    val settings: StateFlow<Settings> = dataStore.settings
        .stateIn(viewModelScope, SharingStarted.Eagerly, dataStore.defaults)

    init {
        val saved = SaveManager.loadFromFile(application)
        if (saved != null) {
            onBoardUpdated(saved.state, saved.playerWhite, newGame = true)
        } else {
            onBoardUpdated(Native.initBoard(), true, newGame = true)
        }

        viewModelScope.launch(Dispatchers.IO) {
            saveFlow.filterNotNull().collect { (playerWhite, game) ->
                SaveManager.saveToFile(getApplication(), playerWhite, game)
            }
        }
    }

    fun resetBoard(playerWhite: Boolean = true) {
        cancelSearch()
        onBoardUpdated(Native.initBoard(), playerWhite, newGame = true)
    }

    fun newGame(playerWhite: Boolean, difficultyLevel: Int) = viewModelScope.launch {
        dataStore.set(SettingsDataStore.DIFFICULTY_LEVEL, difficultyLevel)
        resetBoard(playerWhite)
    }

    fun loadFen(playerWhite: Boolean, fen: String): Boolean {
        val state = Native.loadFen(fen) ?: return false
        cancelSearch()
        onBoardUpdated(state, playerWhite, newGame = true)
        return true
    }

    fun getCurrentFen(): String = Native.getCurrentFen()

    fun exportPgn(): String {
        val date = SimpleDateFormat("yyyy.MM.dd", Locale.US).format(Date())
        return Native.exportPgn(date, stateFlow.value.isPlayerWhite)
    }

    fun showPossibleMoves(square: Int) = stateFlow.update { state ->
        val select = state.selectedSquare != square && state.legalMoves.any { it.from.toInt() == square }
        state.copy(selectedSquare = if (select) square else null)
    }

    fun makeMove(move: Move) {
        val state = Native.makeMove(move.content)
        onBoardUpdated(state)
    }

    fun undo() {
        cancelSearch()
        val state = Native.undo(stateFlow.value.isPlayerWhite) ?: return
        onBoardUpdated(state)
    }

    fun redo() {
        cancelSearch()
        val state = Native.redo(stateFlow.value.isPlayerWhite) ?: return
        onBoardUpdated(state)
    }

    private fun onBoardUpdated(
        state: BoardState,
        playerWhite: Boolean = stateFlow.value.isPlayerWhite,
        newGame: Boolean = false,
    ) {
        stateFlow.update {
            it.copy(
                isPlayerWhite = playerWhite,
                isWhiteTurn = state.isWhiteTurn,
                pieces = state.pieces.map { piece -> IndexedPiece(piece) },
                selectedSquare = null,
                legalMoves = state.legalMoves.map { move -> Move(move) },
                gameState = GameState.getState(state.gameState),
                movesHistory = state.movesHistory.mapIndexed { i, move -> Move(move, state.movesSan[i]) },
                currentMoveIndex = state.currentMoveIndex,
                searchTime = if (newGame) Duration.ZERO else it.searchTime,
                evaluation = state.evalScore,
                uciInfo = if (newGame) "" else it.uciInfo,
            )
        }
        saveFlow.value = playerWhite to Native.saveGame()

        triggerEngineMove()
    }

    fun triggerEngineMove(force: Boolean = false) {
        val state = stateFlow.value
        if (!force && state.isPlayersTurn) return
        if (state.currentMoveIndex != state.movesHistory.lastIndex) return
        if (state.gameState.isGameOver) return
        if (searchJob?.isActive == true) return

        searchJob = viewModelScope.launch {
            setEngineBusy(true)
            val settings = dataStore.settings.first()
            val newState = withContext(Dispatchers.Default) {
                ensureActive()
                Native.engineMove(
                    settings.difficultyLevel,
                    settings.searchTimeSeconds * 1000L,
                    settings.hashSizeMb,
                    settings.threads
                )
            }
            setEngineBusy(false)
            searchJob = null

            if (newState != null) {
                stateFlow.update {
                    it.copy(searchTime = newState.searchTimeMs.milliseconds, uciInfo = newState.uciInfo)
                }
                onBoardUpdated(newState)
                stateFlow.value.lastMove?.let { engineMovesFlow.tryEmit(it) }
            }
        }
    }

    fun stopSearch() {
        Native.stopSearch()
    }

    private fun cancelSearch() {
        searchJob?.cancel()
        Native.stopSearch()
        setEngineBusy(false)
    }

    private fun setEngineBusy(busy: Boolean) = stateFlow.update { it.copy(isEngineBusy = busy) }

    override fun onCleared() {
        searchJob?.cancel()
        Native.stopSearch()
    }
}
