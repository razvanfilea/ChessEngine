package cloud.razvan.chess.wearos.ui.screen

import androidx.compose.animation.graphics.res.animatedVectorResource
import androidx.compose.animation.graphics.res.rememberAnimatedVectorPainter
import androidx.compose.animation.graphics.vector.AnimatedImageVector
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.wear.compose.foundation.CurvedTextStyle
import androidx.wear.compose.foundation.lazy.TransformingLazyColumn
import androidx.wear.compose.foundation.lazy.rememberTransformingLazyColumnState
import androidx.wear.compose.material3.AlertDialog
import androidx.wear.compose.material3.CircularProgressIndicator
import androidx.wear.compose.material3.CircularProgressIndicatorDefaults
import androidx.wear.compose.material3.CompactButton
import androidx.wear.compose.material3.Dialog
import androidx.wear.compose.material3.EdgeButton
import androidx.wear.compose.material3.FilledTonalButton
import androidx.wear.compose.material3.Icon
import androidx.wear.compose.material3.MaterialTheme
import androidx.wear.compose.material3.ScreenScaffold
import androidx.wear.compose.material3.ScrollIndicator
import androidx.wear.compose.material3.SurfaceTransformation
import androidx.wear.compose.material3.Text
import androidx.wear.compose.material3.TimeText
import androidx.wear.compose.material3.lazy.rememberTransformationSpec
import androidx.wear.compose.material3.lazy.transformedHeight
import androidx.wear.compose.material3.timeTextCurvedText
import androidx.wear.compose.material3.timeTextSeparator
import cloud.razvan.chess.common.CapturedPieces
import cloud.razvan.chess.common.model.GameState
import cloud.razvan.chess.common.model.GameUiState
import cloud.razvan.chess.common.ui.ChessBoard
import cloud.razvan.chess.common.ui.MovesHistory
import cloud.razvan.chess.common.ui.VibrateOnEngineMove
import cloud.razvan.chess.common.viewmodel.HomeViewModel
import cloud.razvan.chess.wearos.R
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.milliseconds

object WatchScreen {

    @Composable
    fun Content() {
        val viewModel: HomeViewModel = viewModel()
        val listState = rememberTransformingLazyColumnState()
        val transformationSpec = rememberTransformationSpec()

        var showNewGameDialog by remember { mutableStateOf(false) }

        Dialog(visible = showNewGameDialog, onDismissRequest = { showNewGameDialog = false }) {
            NewGameScreen.Content(viewModel, onDismissRequest = { showNewGameDialog = false })
        }

        val state by viewModel.state.collectAsState()
        val settings by viewModel.settings.collectAsState()
        val contentColor = MaterialTheme.colorScheme.onSurface

        GameFinishedDialog(state.gameState)

        val isThinking = state.isEngineBusy
        val isScreenRound = LocalConfiguration.current.isScreenRound
        val showRing = isThinking && isScreenRound

        KeepScreenOn(isThinking)
        VibrateOnEngineMove(viewModel)

        ScreenScaffold(
            scrollState = listState,
            edgeButton = {
                EdgeButton(onClick = { viewModel.undo() }, enabled = state.canUndo) {
                    Icon(
                        painter = painterResource(R.drawable.ic_undo),
                        contentDescription = stringResource(R.string.action_undo_move)
                    )
                }
            },
            edgeButtonSpacing = 4.dp,
            timeText = { if (settings.showCapturedPieces) MaterialBalance(state) },
            scrollIndicator = if (showRing) null else ({ ScrollIndicator(listState) }),
        ) { contentPadding ->
            TransformingLazyColumn(
                modifier = Modifier.padding(horizontal = 5.dp),
                state = listState,
                contentPadding = contentPadding,
                verticalArrangement = Arrangement.Top,
                horizontalAlignment = Alignment.Start,
            ) {
                item {
                    FilledTonalButton(
                        modifier = Modifier
                            .fillMaxWidth()
                            .transformedHeight(this, transformationSpec),
                        transformation = SurfaceTransformation(transformationSpec),
                        onClick = { showNewGameDialog = true },
                        icon = { Icon(painterResource(R.drawable.ic_new_circle), contentDescription = null) },
                        label = { Text(stringResource(R.string.new_game)) },
                    )
                }

                item {
                    if (isThinking && !isScreenRound) {
                        Box(Modifier.fillMaxWidth()) {
                            Box(Modifier.align(Alignment.TopCenter)) {
                                AnimatedHourGlass()
                            }
                        }

                        Spacer(Modifier.height(2.dp))
                    } else {
                        Spacer(Modifier.height(20.dp))
                    }
                }

                item {
                    MovesHistory(
                        settings.showMovesHistory,
                        state.movesHistory,
                        state.currentMoveIndex,
                        contentColor,
                        modifier = Modifier
                            .graphicsLayer { with(transformationSpec) { applyContainerTransformation(scrollProgress) } }
                            .transformedHeight(this, transformationSpec),
                    )
                }

                item {
                    ChessBoard(
                        modifier = Modifier.padding(bottom = 2.dp),
                        state = state,
                        showCoordinates = settings.showCoordinates,
                        showPossibleMoves = settings.showPossibleMoves,
                        onPieceClick = viewModel::showPossibleMoves,
                        onMove = viewModel::makeMove,
                    )
                }

                if (state.canRedo) {
                    item {
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .transformedHeight(this, transformationSpec)
                        ) {
                            CompactButton(
                                modifier = Modifier.align(Alignment.Center),
                                transformation = SurfaceTransformation(transformationSpec),
                                onClick = { viewModel.redo() },
                                icon = { Icon(painterResource(R.drawable.ic_redo), contentDescription = null) },
                                label = { Text(stringResource(R.string.action_redo_move)) },
                            )
                        }
                    }
                }
            }

            if (showRing) {
                CircularProgressIndicator(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(CircularProgressIndicatorDefaults.FullScreenPadding),
                )
            }
        }
    }

    @Composable
    private fun MaterialBalance(state: GameUiState) {
        val (mine, theirs) = remember(state.pieces, state.isPlayerWhite) {
            val captured = CapturedPieces.from(state.pieces)
            val white = capturedText(captured.capturedByWhite, captured.whiteScore)
            val black = capturedText(captured.capturedByBlack, captured.blackScore)
            if (state.isPlayerWhite) white to black else black to white
        }
        if (mine.isEmpty() && theirs.isEmpty()) return

        val theirsStyle = CurvedTextStyle(color = MaterialTheme.colorScheme.onSurfaceVariant)
        TimeText {
            if (mine.isNotEmpty()) timeTextCurvedText(mine)
            if (mine.isNotEmpty() && theirs.isNotEmpty()) timeTextSeparator()
            if (theirs.isNotEmpty()) timeTextCurvedText(theirs, theirsStyle)
        }
    }

    private const val PIECE_SYMBOLS = "♟♞♝♜♛♚"

    private fun capturedText(pieces: List<Byte>, score: Int) = buildString {
        // U+FE0E: draw the pawn as text, not as an emoji
        pieces.forEach { append(PIECE_SYMBOLS[it.toInt()]).append('︎') }
        if (score > 0) append(" +").append(score)
    }

    @Composable
    private fun KeepScreenOn(enabled: Boolean) {
        val view = LocalView.current
        DisposableEffect(enabled) {
            view.keepScreenOn = enabled
            onDispose { view.keepScreenOn = false }
        }
    }


    @Composable
    private fun AnimatedHourGlass() {
        var atEnd by remember { mutableStateOf(false) }

        val icon = AnimatedImageVector.animatedVectorResource(R.drawable.ic_animated_hourglass)

        Icon(
            painter = rememberAnimatedVectorPainter(icon, atEnd),
            modifier = Modifier.size(18.dp),
            tint = MaterialTheme.colorScheme.onSurface,
            contentDescription = null,
        )

        LaunchedEffect(Unit) {
            while (true) {
                delay(250.milliseconds)
                atEnd = !atEnd
                delay(800.milliseconds)
            }
        }
    }
}

@Composable
private fun GameFinishedDialog(gameState: GameState) {
    val messageRes = when (gameState) {
        GameState.WINNER_WHITE -> R.string.victory_white
        GameState.WINNER_BLACK -> R.string.victory_black
        GameState.DRAW -> R.string.draw
        else -> null
    }

    var dismissed by remember(gameState) { mutableStateOf(false) }

    AlertDialog(
        visible = messageRes != null && !dismissed,
        onDismissRequest = { dismissed = true },
        title = { messageRes?.let { Text(stringResource(it)) } },
        edgeButton = {
            EdgeButton(onClick = { dismissed = true }) {
                Icon(
                    painter = painterResource(R.drawable.ic_done),
                    contentDescription = stringResource(android.R.string.ok)
                )
            }
        }
    )
}
