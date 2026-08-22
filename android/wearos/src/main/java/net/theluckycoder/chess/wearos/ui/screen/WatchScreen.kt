package net.theluckycoder.chess.wearos.ui.screen

import androidx.compose.animation.graphics.ExperimentalAnimationGraphicsApi
import androidx.compose.animation.graphics.res.animatedVectorResource
import androidx.compose.animation.graphics.res.rememberAnimatedVectorPainter
import androidx.compose.animation.graphics.vector.AnimatedImageVector
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.wear.compose.material.Button
import androidx.wear.compose.material.ButtonDefaults
import androidx.wear.compose.material.Icon
import androidx.wear.compose.material.MaterialTheme
import androidx.wear.compose.material.Scaffold
import androidx.wear.compose.material.Text
import androidx.wear.compose.material.Vignette
import androidx.wear.compose.material.VignettePosition
import androidx.wear.compose.material.dialog.Alert
import androidx.wear.compose.material.dialog.Dialog
import androidx.wear.compose.material.itemsIndexed
import kotlinx.coroutines.delay
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.Move
import net.theluckycoder.chess.common.ui.ChessBoard
import net.theluckycoder.chess.common.viewmodel.HomeViewModel
import net.theluckycoder.chess.wearos.R
import net.theluckycoder.chess.wearos.ui.TextIconButton
import net.theluckycoder.chess.wearos.ui.isScreenRound

object WatchScreen {

    @Composable
    fun Content() {
        val viewModel: HomeViewModel = viewModel()
        val listState = rememberLazyListState()

        var showNewGameDialog by remember { mutableStateOf(false) }

        Dialog(onDismissRequest = { showNewGameDialog = false }, showDialog = showNewGameDialog) {
            NewGameScreen.Content(onDismissRequest = { showNewGameDialog = false })
        }

        val isThinking by viewModel.isEngineBusy.collectAsState()

        Scaffold(
            vignette = {
                Vignette(vignettePosition = VignettePosition.Top)
            }
        ) {
            LazyColumn(
                modifier = Modifier
                    .background(MaterialTheme.colorScheme.background)
                    .padding(horizontal = 5.dp),
                state = listState,
            ) {
                item {
                    Spacer(Modifier.height(32.dp))
                }

                item {
                    Box(Modifier.fillMaxWidth()) {
                        TextIconButton(
                            modifier = Modifier.align(Alignment.Center),
                            onClick = { showNewGameDialog = true },
                            text = stringResource(R.string.new_game)
                        ) {
                            Icon(
                                painter = painterResource(R.drawable.ic_new_circle),
                                contentDescription = null
                            )
                        }
                    }
                }

                item {
                    if (isThinking) {
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
                    WatchChessBoard(viewModel)
                }

                item {
                    UndoRedoActions(viewModel)
                }

                /*item {
                    SettingsActions()
                }*/

                item {
                    Spacer(Modifier.padding(16.dp))
                }
            }
        }
    }

    @OptIn(ExperimentalAnimationGraphicsApi::class)
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
            delay(250)
            atEnd = true
        }
    }
}

@Composable
private fun WatchChessBoard(
    viewModel: HomeViewModel = viewModel()
) {
    val isPlayerWhite by viewModel.playerPlayingWhite.collectAsState()
    val tiles by viewModel.tiles.collectAsState()
    val pieces by viewModel.pieces.collectAsState()
    val gameState by viewModel.gameState.collectAsState()

    val showPromotionDialog = remember { mutableStateOf(emptyList<Move>()) }

    ChessBoard(
        modifier = Modifier
            .padding(top = 8.dp, bottom = if (isScreenRound()) 8.dp else 2.dp),
        isPlayerWhite = isPlayerWhite,
        tiles = tiles,
        pieces = pieces,
        gameState = gameState,
        onPieceClick = { viewModel.showPossibleMoves(it.square) },
        onShowPromotion = { showPromotionDialog.value = it }
    )

    if (showPromotionDialog.value.isNotEmpty())
        PromotionDialog(showPromotionDialog)
}

@Composable
private fun PromotionDialog(showPromotionDialog: MutableState<List<Move>>) {
    val promotionResources = remember {
        intArrayOf(
            R.drawable.w_queen, R.drawable.w_rook,
            R.drawable.w_knight, R.drawable.w_bishop,
        )
    }
    val piecesPainters = promotionResources.map { painterResource(id = it) }

    Alert(
        title = {
            Text(stringResource(id = R.string.promotion_choose_piece))
        },
    ) {
//        TODO()
        itemsIndexed(piecesPainters) { index, painter ->
            Button(
                colors = ButtonDefaults.secondaryButtonColors(),
                onClick = {
                    Native.makeMove(showPromotionDialog.value[index])
                    showPromotionDialog.value = emptyList()
                }
            ) {
                Image(
                    modifier = Modifier,
                    painter = painter, contentDescription = null
                )
            }
        }

        item {
            Button(
                modifier = Modifier.fillMaxWidth(),
                onClick = { showPromotionDialog.value = emptyList() }
            ) {
                Icon(
                    modifier = Modifier.fillMaxWidth(),
                    painter = painterResource(id = net.theluckycoder.chess.common.R.drawable.ic_close),
                    contentDescription = stringResource(id = android.R.string.cancel)
                )
            }
        }
    }
}

@Composable
private fun UndoRedoActions(
    viewModel: HomeViewModel = viewModel(),
) = Row(
    modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 16.dp, vertical = 8.dp),
    horizontalArrangement = Arrangement.SpaceBetween
) {
    val movesIndex by viewModel.currentMoveIndex.collectAsState()
    val movesHistory by viewModel.movesHistory.collectAsState()

    TextIconButton(
        onClick = { Native.undoMoves() },
        enabled = movesIndex >= 0,
        text = stringResource(R.string.action_undo_move),
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_undo),
            contentDescription = stringResource(R.string.action_undo_move)
        )
    }

    TextIconButton(
        onClick = { Native.redoMoves() },
        enabled = movesIndex != movesHistory.lastIndex,
        text = stringResource(R.string.action_redo_move)
    ) {
        Icon(
            painter = painterResource(id = R.drawable.ic_redo),
            contentDescription = stringResource(id = R.string.action_redo_move)
        )
    }
}

/*@Composable
private fun SettingsActions() = Row(
    modifier = Modifier
        .fillMaxWidth()
        .padding(horizontal = 16.dp, vertical = 8.dp),
    horizontalArrangement = Arrangement.SpaceBetween
) {
    val navigator = LocalNavigator.currentOrThrow

    TextIconButton(
        onClick = { navigator.push(NewGameScreen()) },
        text = stringResource(R.string.new_game)
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_new_circle),
            contentDescription = stringResource(R.string.new_game)
        )
    }

    TextIconButton(
        onClick = { navigator.push(SettingsScreen()) },
        text = stringResource(R.string.title_settings),
    ) {
        Icon(
            painter = painterResource(R.drawable.ic_settings),
            contentDescription = stringResource(R.string.title_settings)
        )
    }
}*/
