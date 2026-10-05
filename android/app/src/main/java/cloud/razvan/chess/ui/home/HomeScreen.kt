package cloud.razvan.chess.ui.home

import android.content.Intent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.graphics.res.animatedVectorResource
import androidx.compose.animation.graphics.res.rememberAnimatedVectorPainter
import androidx.compose.animation.graphics.vector.AnimatedImageVector
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.delay
import cloud.razvan.chess.R
import cloud.razvan.chess.common.ui.CapturedPiecesLists
import cloud.razvan.chess.common.ui.ChessBoard
import cloud.razvan.chess.common.ui.MovesHistory
import cloud.razvan.chess.common.ui.VibrateOnEngineMove
import cloud.razvan.chess.common.viewmodel.HomeViewModel
import cloud.razvan.chess.ui.preferences.PreferencesActivity
import kotlin.time.Duration.Companion.milliseconds

@Composable
fun HomeScreen(
    viewModel: HomeViewModel = viewModel()
) {
    val state by viewModel.state.collectAsState()

    val showNewGameDialog = rememberSaveable { mutableStateOf(false) }
    val showPositionDialog = rememberSaveable { mutableStateOf(false) }

    HomeDialogs(showNewGameDialog, showPositionDialog, state.gameState, viewModel)
    VibrateOnEngineMove(viewModel)

    val onNewGame = { showNewGameDialog.value = true }
    val onPosition = { showPositionDialog.value = true }

    BoxWithConstraints(Modifier.fillMaxSize()) {
        if (maxWidth > maxHeight)
            WideLayout(onNewGame, onPosition, viewModel)
        else
            TallLayout(onNewGame, onPosition, viewModel)
    }
}

@Composable
private fun TallLayout(onNewGame: () -> Unit, onPosition: () -> Unit, viewModel: HomeViewModel) {
    val state by viewModel.state.collectAsState()
    val settings by viewModel.settings.collectAsState()
    val contentColor = MaterialTheme.colorScheme.onSurface

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        topBar = {
            Column(modifier = Modifier.fillMaxWidth()) {
                TopBar(viewModel)
                MovesHistory(settings.showMovesHistory, state.movesHistory, state.currentMoveIndex, contentColor)
            }
        },
        bottomBar = { ActionsBar(Modifier.navigationBarsPadding(), onNewGame, onPosition, viewModel) }
    ) { padding ->
        Box(
            Modifier
                .fillMaxSize()
                .padding(padding),
            contentAlignment = Alignment.Center,
        ) {
            CapturedPiecesLists(
                show = settings.showCapturedPieces,
                isPlayerWhite = state.isPlayerWhite,
                pieces = state.pieces,
                contentColor = contentColor,
            ) {
                ChessBoard(
                    state = state,
                    showCoordinates = settings.showCoordinates,
                    showPossibleMoves = settings.showPossibleMoves,
                    onPieceClick = viewModel::showPossibleMoves,
                    onMove = viewModel::makeMove,
                )
            }
        }
    }
}

@Composable
private fun WideLayout(onNewGame: () -> Unit, onPosition: () -> Unit, viewModel: HomeViewModel) {
    val state by viewModel.state.collectAsState()
    val settings by viewModel.settings.collectAsState()
    val contentColor = MaterialTheme.colorScheme.onSurface

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        contentWindowInsets = WindowInsets.safeDrawing,
    ) { padding ->
        Row(
            Modifier
                .padding(padding)
                .fillMaxSize(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ChessBoard(
                state = state,
                showCoordinates = settings.showCoordinates,
                showPossibleMoves = settings.showPossibleMoves,
                onPieceClick = viewModel::showPossibleMoves,
                onMove = viewModel::makeMove,
            )

            Column(
                modifier = Modifier
                    .fillMaxHeight()
                    .weight(1f)
            ) {
                TopBar(viewModel, windowInsets = WindowInsets(0))
                MovesHistory(settings.showMovesHistory, state.movesHistory, state.currentMoveIndex, contentColor)

                CapturedPiecesLists(
                    modifier = Modifier.weight(1f),
                    show = settings.showCapturedPieces,
                    isPlayerWhite = state.isPlayerWhite,
                    pieces = state.pieces,
                    contentColor = contentColor,
                ) {
                    Spacer(Modifier.weight(1f))
                }

                ActionsBar(Modifier.padding(bottom = 4.dp), onNewGame, onPosition, viewModel)
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TopBar(
    viewModel: HomeViewModel,
    windowInsets: WindowInsets = TopAppBarDefaults.windowInsets,
) = TopAppBar(
    title = {
        val state by viewModel.state.collectAsState()

        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(stringResource(id = R.string.app_name))
            EngineBusyIndicator(state.isEngineBusy)
        }
    },
    actions = { DebugMenu(viewModel) },
    windowInsets = windowInsets,
)

@Composable
private fun EngineBusyIndicator(visible: Boolean) = AnimatedVisibility(
    visible = visible,
    enter = fadeIn(),
    exit = fadeOut(),
) {
    var atEnd by remember { mutableStateOf(false) }
    val icon = AnimatedImageVector.animatedVectorResource(R.drawable.ic_animated_hourglass)

    Icon(
        painter = rememberAnimatedVectorPainter(icon, atEnd),
        modifier = Modifier.size(24.dp),
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

@Composable
private fun DebugMenu(viewModel: HomeViewModel) {
    val state by viewModel.state.collectAsState()
    val settings by viewModel.settings.collectAsState()
    var showActionsMenu by rememberSaveable { mutableStateOf(false) }

    if (!settings.showDebugBasic) return

    IconButton(onClick = { showActionsMenu = true }) {
        Icon(
            painter = painterResource(id = R.drawable.ic_more_options_vertical),
            contentDescription = stringResource(id = R.string.action_more_options)
        )

        DropdownMenu(
            expanded = showActionsMenu,
            onDismissRequest = { showActionsMenu = false }
        ) {
            DropdownMenuItem(
                onClick = {
                    showActionsMenu = false
                    viewModel.triggerEngineMove(force = true)
                },
                text = { Text(text = stringResource(id = R.string.action_make_engine_move)) })

            if (state.isEngineBusy) {
                DropdownMenuItem(onClick = {
                    showActionsMenu = false
                    viewModel.stopSearch()
                }, text = { Text(text = stringResource(id = R.string.action_stop_search)) })
            }
        }
    }
}

@Composable
private fun ActionsBar(
    modifier: Modifier,
    onNewGame: () -> Unit,
    onPosition: () -> Unit,
    viewModel: HomeViewModel
) = Column(
    modifier = modifier,
    verticalArrangement = Arrangement.Bottom,
    horizontalAlignment = Alignment.CenterHorizontally,
) {
    val state by viewModel.state.collectAsState()
    val settings by viewModel.settings.collectAsState()

    if (settings.showDebugBasic) {
        val text = buildString {
            append(stringResource(R.string.debug_stats, state.searchTime.toString(), state.evaluation))
            if (settings.showDebugAdvanced && state.uciInfo.isNotEmpty()) {
                append('\n').append(state.uciInfo)
            }
        }

        Text(
            text = text,
            fontSize = 13.5.sp,
        )
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        IconButton(
            onClick = { viewModel.undo() },
            enabled = state.canUndo,
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_undo),
                contentDescription = stringResource(id = R.string.action_undo_move)
            )
        }

        IconButton(
            onClick = { viewModel.redo() },
            enabled = state.canRedo,
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_redo),
                contentDescription = stringResource(id = R.string.action_redo_move)
            )
        }

        IconButton(
            onClick = onNewGame
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_new_circle),
                contentDescription = stringResource(id = R.string.new_game)
            )
        }

        IconButton(
            onClick = onPosition
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_share),
                contentDescription = stringResource(id = R.string.fen_pgn_position)
            )
        }

        val context = LocalContext.current
        IconButton(
            onClick = { context.startActivity(Intent(context, PreferencesActivity::class.java)) }
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_settings),
                contentDescription = stringResource(id = R.string.title_settings)
            )
        }
    }
}
