package net.theluckycoder.chess.ui.home

import android.widget.Toast
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import net.theluckycoder.chess.R
import net.theluckycoder.chess.common.Pgn
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.GameState
import net.theluckycoder.chess.common.ui.ChooseSidesToggle
import net.theluckycoder.chess.common.viewmodel.HomeViewModel
import kotlin.math.roundToInt
import kotlin.random.Random

@Composable
fun HomeDialogs(viewModel: HomeViewModel = viewModel()) {

    if (viewModel.showNewGameDialog.value)
        NewGameDialog(viewModel)

    if (viewModel.showShareDialog.value)
        SharePositionDialog(viewModel)

    if (viewModel.showImportDialog.value)
        ImportPositionDialog(viewModel)

    val gameState by viewModel.gameState.collectAsState()
    when (gameState) {
        GameState.DRAW, GameState.WINNER_WHITE, GameState.WINNER_BLACK -> {
            GameFinishedDialog(gameState)
        }

        else -> Unit
    }
}

@Preview
@Composable
private fun NewGameDialog(viewModel: HomeViewModel = viewModel()) {
    val sidesToggleIndex = remember { mutableStateOf(0) }
    var difficultyLevel by remember { mutableStateOf(1f) }

    AlertDialog(
        onDismissRequest = { viewModel.showNewGameDialog.value = false },
        title = { Text(stringResource(id = R.string.new_game)) },
        text = {
            Column(modifier = Modifier.fillMaxWidth()) {
                Text(
                    text = stringResource(id = R.string.side),
                    color = MaterialTheme.colorScheme.secondary
                )

                ChooseSidesToggle(sidesToggleIndex = sidesToggleIndex)

                Text(
                    modifier = Modifier.padding(top = 16.dp),
                    text = stringResource(R.string.difficulty_level, difficultyLevel.roundToInt()),
                    color = MaterialTheme.colorScheme.secondary
                )

                Slider(
                    modifier = Modifier.padding(8.dp),
                    value = difficultyLevel,
                    onValueChange = { difficultyLevel = it },
                    valueRange = 1f..8f,
                    steps = 6,
                )
            }
        },
        confirmButton = {
            TextButton(onClick = {
                val playerWhite = when (sidesToggleIndex.value) {
                    0 -> true
                    1 -> false
                    else -> Random.nextBoolean()
                }

                val level = difficultyLevel.roundToInt()
                viewModel.updateDifficulty(level)
                viewModel.showNewGameDialog.value = false

                // Restart the Game
                viewModel.resetBoard(playerWhite)

            }) {
                Text(text = stringResource(id = R.string.action_start))
            }
        },
        dismissButton = {
            TextButton(onClick = {
                viewModel.showNewGameDialog.value = false
            }) {
                Text(text = stringResource(id = android.R.string.cancel))
            }
        }
    )
}

@Composable
private fun SharePositionDialog(viewModel: HomeViewModel = viewModel()) {
    val currentFen = remember { Native.getCurrentFen() }
    val pgn = remember {
        Pgn.export(
            viewModel.playerPlayingWhite.value,
            Native.getStartFen(),
            viewModel.movesHistory.value,
            viewModel.gameState.value
        )
    }

    AlertDialog(
        onDismissRequest = { viewModel.showShareDialog.value = false },
        title = { Text(stringResource(id = R.string.fen_pgn_position_share)) },
        text = {
            Column(Modifier.fillMaxWidth()) {
                Text(
                    text = stringResource(id = R.string.fen_position_current),
                    color = MaterialTheme.colorScheme.secondary
                )
                Text(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 6.dp),
                    text = currentFen,
                    fontSize = 14.5.sp
                )
            }
        },
        dismissButton = {
            TextButton(
                onClick = { viewModel.showShareDialog.value = false }
            ) {
                Text(text = stringResource(id = R.string.action_close))
            }
        },
        confirmButton = {
            val context = LocalContext.current
            val clipboardManager = LocalClipboardManager.current

            Row {
                TextButton(
                    onClick = {
                        clipboardManager.setText(AnnotatedString(currentFen))
                        Toast.makeText(context, R.string.fen_position_copied, Toast.LENGTH_SHORT)
                            .show()
                        viewModel.showShareDialog.value = false
                    }
                ) {
                    Text(text = stringResource(id = R.string.action_fen_copy))
                }

                TextButton(
                    onClick = {
                        clipboardManager.setText(AnnotatedString(pgn))
                        Toast.makeText(context, R.string.pgn_position_copied, Toast.LENGTH_SHORT)
                            .show()
                        viewModel.showShareDialog.value = false
                    }
                ) {
                    Text(text = stringResource(id = R.string.action_pgn_copy))
                }
            }

        }
    )
}

@Composable
private fun ImportPositionDialog(viewModel: HomeViewModel = viewModel()) {
    var newFen by remember { mutableStateOf("") }
    val sidesToggleIndex = remember { mutableIntStateOf(0) }
    var failedToLoad by remember { mutableStateOf(false) }

    AlertDialog(
        onDismissRequest = { viewModel.showImportDialog.value = false },
        title = { Text(stringResource(id = R.string.fen_position_load)) },
        text = {
            Column(Modifier.fillMaxWidth()) {
                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    value = newFen,
                    onValueChange = { newFen = it },
                    placeholder = { Text(text = "FEN") },
                    isError = failedToLoad
                )

                Text(
                    modifier = Modifier.padding(top = 8.dp, bottom = 4.dp),
                    text = stringResource(id = R.string.side),
                    color = MaterialTheme.colorScheme.secondary
                )

                ChooseSidesToggle(sidesToggleIndex = sidesToggleIndex)
            }
        },
        dismissButton = {
            TextButton(
                onClick = { viewModel.showImportDialog.value = false }
            ) {
                Text(text = stringResource(id = R.string.action_close))
            }
        },
        confirmButton = {
            val context = LocalContext.current
            Button(
                onClick = {
                    if (newFen.isBlank()) {
                        Toast.makeText(
                            context,
                            R.string.fen_position_error_empty,
                            Toast.LENGTH_SHORT
                        ).show()
                        return@Button
                    }

                    val playerWhite = when (sidesToggleIndex.intValue) {
                        0 -> true
                        1 -> false
                        else -> Random.nextBoolean()
                    }

                    if (Native.loadFen(playerWhite, newFen)) {
                        failedToLoad = false
                        viewModel.showImportDialog.value = false
                        Toast.makeText(
                            context,
                            R.string.fen_position_loaded,
                            Toast.LENGTH_SHORT
                        ).show()
                    } else {
                        Toast.makeText(context, R.string.fen_position_error, Toast.LENGTH_LONG)
                            .show()
                        failedToLoad = true
                    }
                }
            ) {
                Text(text = stringResource(id = R.string.action_load))
            }
        }
    )
}

@Composable
private fun GameFinishedDialog(gameState: GameState) {
    val messageRes = when (gameState) {
        GameState.WINNER_WHITE -> R.string.victory_white
        GameState.WINNER_BLACK -> R.string.victory_black
        GameState.DRAW -> R.string.draw
        GameState.INVALID -> R.string.invalid_game
        else -> return
    }

    var showDialog by remember { mutableStateOf(true) }

    if (!showDialog) return

    AlertDialog(
        onDismissRequest = { showDialog = false },
        title = { Text(stringResource(id = messageRes)) },
        confirmButton = {
            TextButton(onClick = { showDialog = false }) {
                Text(text = stringResource(id = android.R.string.ok))
            }
        }
    )
}
