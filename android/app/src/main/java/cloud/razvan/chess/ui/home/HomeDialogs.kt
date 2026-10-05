package cloud.razvan.chess.ui.home

import android.content.ClipData
import android.widget.Toast
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import cloud.razvan.chess.R
import cloud.razvan.chess.common.SettingsDataStore
import cloud.razvan.chess.common.model.GameState
import cloud.razvan.chess.common.ui.ChooseSidesToggle
import cloud.razvan.chess.common.ui.Side
import cloud.razvan.chess.common.viewmodel.HomeViewModel
import kotlin.math.roundToInt

@Composable
fun HomeDialogs(
    showNewGameDialog: MutableState<Boolean>,
    showPositionDialog: MutableState<Boolean>,
    gameState: GameState,
    viewModel: HomeViewModel
) {

    if (showNewGameDialog.value)
        NewGameDialog(showNewGameDialog, viewModel)

    if (showPositionDialog.value)
        PositionDialog(showPositionDialog, viewModel)

    if (gameState.isGameOver)
        GameFinishedDialog(gameState)
}

@Composable
private fun NewGameDialog(show: MutableState<Boolean>, viewModel: HomeViewModel) {
    val selectedSide = remember { mutableStateOf(Side.WHITE) }
    var difficultyLevel by remember { mutableFloatStateOf(viewModel.settings.value.difficultyLevel.toFloat()) }
    val levels = SettingsDataStore.DIFFICULTY_LEVELS

    AlertDialog(
        onDismissRequest = { show.value = false },
        title = { Text(stringResource(id = R.string.new_game)) },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
            ) {
                Text(
                    text = stringResource(id = R.string.side),
                    color = MaterialTheme.colorScheme.secondary
                )

                ChooseSidesToggle(
                    selectedSide = selectedSide,
                    primaryColor = MaterialTheme.colorScheme.primary
                )

                Text(
                    modifier = Modifier.padding(top = 16.dp),
                    text = stringResource(R.string.difficulty_level, difficultyLevel.roundToInt()),
                    color = MaterialTheme.colorScheme.secondary
                )

                Slider(
                    modifier = Modifier.padding(8.dp),
                    value = difficultyLevel,
                    onValueChange = { difficultyLevel = it },
                    valueRange = levels.first.toFloat()..levels.last.toFloat(),
                    steps = levels.count() - 2,
                )
            }
        },
        confirmButton = {
            TextButton(onClick = {
                show.value = false
                viewModel.newGame(selectedSide.value.isPlayerWhite(), difficultyLevel.roundToInt())

            }) {
                Text(text = stringResource(id = R.string.action_start))
            }
        },
        dismissButton = {
            TextButton(onClick = {
                show.value = false
            }) {
                Text(text = stringResource(id = android.R.string.cancel))
            }
        }
    )
}

@Composable
private fun PositionDialog(show: MutableState<Boolean>, viewModel: HomeViewModel) {
    val currentFen = remember { viewModel.getCurrentFen() }
    val pgn = remember { viewModel.exportPgn() }
    var newFen by rememberSaveable { mutableStateOf("") }
    val selectedSide = remember { mutableStateOf(Side.WHITE) }
    var failedToLoad by remember { mutableStateOf(false) }
    val context = LocalContext.current

    AlertDialog(
        onDismissRequest = { show.value = false },
        title = { Text(stringResource(id = R.string.fen_pgn_position)) },
        text = {
            val clipboard = LocalClipboard.current
            val scope = rememberCoroutineScope()
            fun copy(text: String, toastRes: Int) {
                scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText(text, text))) }
                Toast.makeText(context, toastRes, Toast.LENGTH_SHORT).show()
                show.value = false
            }

            Column(
                Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
            ) {
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

                Row(Modifier.align(Alignment.End)) {
                    TextButton(onClick = { copy(currentFen, R.string.fen_position_copied) }) {
                        Text(text = stringResource(id = R.string.action_fen_copy))
                    }

                    TextButton(onClick = { copy(pgn, R.string.pgn_position_copied) }) {
                        Text(text = stringResource(id = R.string.action_pgn_copy))
                    }
                }

                HorizontalDivider(Modifier.padding(vertical = 8.dp))

                Text(
                    modifier = Modifier.padding(bottom = 6.dp),
                    text = stringResource(id = R.string.fen_position_load),
                    color = MaterialTheme.colorScheme.secondary
                )

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

                ChooseSidesToggle(
                    selectedSide = selectedSide,
                    primaryColor = MaterialTheme.colorScheme.primary
                )
            }
        },
        dismissButton = {
            TextButton(
                onClick = { show.value = false }
            ) {
                Text(text = stringResource(id = R.string.action_close))
            }
        },
        confirmButton = {
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

                    if (viewModel.loadFen(selectedSide.value.isPlayerWhite(), newFen)) {
                        failedToLoad = false
                        show.value = false
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
        else -> return
    }

    var showDialog by rememberSaveable(gameState) { mutableStateOf(true) }

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
