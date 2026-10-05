package cloud.razvan.chess.wearos.ui.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.wear.compose.foundation.lazy.TransformingLazyColumn
import androidx.wear.compose.foundation.lazy.rememberTransformingLazyColumnState
import androidx.wear.compose.material3.EdgeButton
import androidx.wear.compose.material3.Icon
import androidx.wear.compose.material3.ListHeader
import androidx.wear.compose.material3.ListSubHeader
import androidx.wear.compose.material3.RadioButton
import androidx.wear.compose.material3.ScreenScaffold
import androidx.wear.compose.material3.Slider
import androidx.wear.compose.material3.SurfaceTransformation
import androidx.wear.compose.material3.Text
import androidx.wear.compose.material3.lazy.rememberTransformationSpec
import androidx.wear.compose.material3.lazy.transformedHeight
import cloud.razvan.chess.common.SettingsDataStore
import cloud.razvan.chess.common.ui.Side
import cloud.razvan.chess.common.viewmodel.HomeViewModel
import cloud.razvan.chess.wearos.R

object NewGameScreen {

    @Composable
    fun Content(viewModel: HomeViewModel, onDismissRequest: () -> Unit) {
        val listState = rememberTransformingLazyColumnState()
        val transformationSpec = rememberTransformationSpec()

        var selectedSide by remember { mutableStateOf(Side.WHITE) }
        var difficultyLevel by remember { mutableIntStateOf(viewModel.settings.value.difficultyLevel) }

        ScreenScaffold(
            scrollState = listState,
            edgeButton = {
                EdgeButton(onClick = {
                    viewModel.newGame(selectedSide.isPlayerWhite(), difficultyLevel)
                    onDismissRequest()
                }) {
                    Icon(
                        painter = painterResource(R.drawable.ic_done),
                        contentDescription = stringResource(R.string.new_game)
                    )
                }
            }
        ) { contentPadding ->
            TransformingLazyColumn(
                state = listState,
                contentPadding = contentPadding
            ) {
                item {
                    ListHeader(
                        modifier = Modifier.transformedHeight(this, transformationSpec),
                        transformation = SurfaceTransformation(transformationSpec),
                    ) {
                        Text(stringResource(R.string.new_game))
                    }
                }

                item {
                    ListSubHeader(
                        modifier = Modifier.transformedHeight(this, transformationSpec),
                        transformation = SurfaceTransformation(transformationSpec),
                    ) {
                        Text(stringResource(R.string.side))
                    }
                }

                items(Side.entries.size) { index ->
                    val side = Side.entries[index]
                    RadioButton(
                        modifier = Modifier
                            .fillMaxWidth()
                            .transformedHeight(this, transformationSpec),
                        transformation = SurfaceTransformation(transformationSpec),
                        selected = selectedSide == side,
                        onSelect = { selectedSide = side },
                        icon = {
                            Image(
                                modifier = Modifier.size(24.dp),
                                painter = painterResource(side.painterRes),
                                contentDescription = null,
                            )
                        },
                        label = { Text(stringResource(side.contentDescriptionRes)) },
                    )
                }

                item {
                    ListSubHeader(
                        modifier = Modifier.transformedHeight(this, transformationSpec),
                        transformation = SurfaceTransformation(transformationSpec),
                    ) {
                        Text(stringResource(R.string.difficulty_level, difficultyLevel))
                    }
                }

                item {
                    Slider(
                        modifier = Modifier
                            .graphicsLayer { with(transformationSpec) { applyContainerTransformation(scrollProgress) } }
                            .transformedHeight(this, transformationSpec),
                        value = difficultyLevel,
                        onValueChange = { difficultyLevel = it },
                        valueProgression = SettingsDataStore.DIFFICULTY_LEVELS,
                    )
                }
            }
        }
    }
}
