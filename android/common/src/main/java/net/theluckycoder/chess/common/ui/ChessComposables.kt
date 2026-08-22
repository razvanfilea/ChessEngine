package net.theluckycoder.chess.common.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ExperimentalAnimationApi
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.expandIn
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconToggleButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import net.theluckycoder.chess.common.CapturedPieces
import net.theluckycoder.chess.common.R
import net.theluckycoder.chess.common.isIndexVisible
import net.theluckycoder.chess.common.model.Move
import net.theluckycoder.chess.common.model.Piece
import net.theluckycoder.chess.common.viewmodel.HomeViewModel

@Composable
fun ChooseSidesToggle(
    modifier: Modifier = Modifier,
    sidesToggleIndex: MutableState<Int>,
    primaryColor: Color = MaterialTheme.colorScheme.primary,
) {
    class Side(val painterRes: Int, val backgroundColorRes: Int, val contentDescriptionRes: Int)

    val sides = listOf(
        Side(R.drawable.w_king, R.color.side_white, R.string.side_white),
        Side(R.drawable.b_king, R.color.side_black, R.string.side_black),
        Side(R.drawable.side_random, R.color.side_random, R.string.side_random),
    )

    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(8.dp)
            .clip(RoundedCornerShape(8.dp))
    ) {
        sides.forEachIndexed { index, side ->
            val backgroundColor = if (sidesToggleIndex.value == index)
                primaryColor.copy(alpha = 0.5f)
            else
                colorResource(id = side.backgroundColorRes)

            IconToggleButton(
                modifier = Modifier
                    .background(backgroundColor)
                    .weight(1f)
                    .padding(4.dp),
                checked = sidesToggleIndex.value == index,
                onCheckedChange = { sidesToggleIndex.value = index }
            ) {
                Icon(
                    modifier = Modifier.size(54.dp),
                    painter = painterResource(id = side.painterRes),
                    tint = Color.Unspecified,
                    contentDescription = stringResource(id = side.contentDescriptionRes),
                )
            }
        }
    }
}

@OptIn(ExperimentalAnimationApi::class)
@Composable
fun MovesHistory(
    show: Boolean,
    movesHistory: List<Move>,
    currentMoveIndex: Int,
    textColor: Color = Color.Unspecified,
    backgroundColor: Color = Color(0xFF222222),
) {
    val transition =
        remember { MutableTransitionState(initialState = false) }.apply { targetState = show }

    AnimatedVisibility(
        visibleState = transition,
        modifier = Modifier,
        enter = fadeIn() + expandIn(expandFrom = Alignment.TopCenter),
        exit = shrinkOut(shrinkTowards = Alignment.TopCenter) + fadeOut()
    ) {
        val listState = rememberLazyListState()

        LaunchedEffect(movesHistory, currentMoveIndex) {
            if (movesHistory.isNotEmpty()
                && currentMoveIndex in movesHistory.indices
                && !listState.isIndexVisible(currentMoveIndex)
            ) listState.animateScrollToItem(currentMoveIndex)
        }

        LazyRow(
            state = listState,
            modifier = Modifier
                .fillMaxWidth()
                .background(backgroundColor)
                .padding(4.dp),
            content = {
                if (movesHistory.isNotEmpty()) {
                    itemsIndexed(movesHistory) { index, item ->
                        val padding = if (index % 2 == 0)
                            Modifier.padding(start = 6.dp, end = 2.dp)
                        else
                            Modifier.padding(start = 2.dp, end = 6.dp)

                        Row(
                            modifier = padding
                        ) {
                            if (index % 2 == 0) {
                                Text(
                                    text = "${index / 2 + 1}. ",
                                    color = Color.Gray,
                                    fontSize = 13.sp,
                                )
                            }

                            val modifier = Modifier
                                .padding(1.dp)
                                .then(
                                    if (currentMoveIndex == index)
                                        Modifier
                                            .clip(RoundedCornerShape(2.dp))
                                            .background(Color.Gray)
                                    else Modifier
                                )
                            Text(
                                modifier = modifier,
                                text = item.toString(),
                                color = textColor,
                                fontSize = 13.sp,
                            )
                        }
                    }
                } else
                    item { Text(modifier = Modifier.padding(1.dp), text = "", fontSize = 13.sp) }
            }
        )
    }
}

@Composable
fun CapturedPiecesLists(
    modifier: Modifier = Modifier,
    show: Boolean,
    content: @Composable ColumnScope.() -> Unit
) = Column(modifier) {
    val viewModel = viewModel<HomeViewModel>()

    val isPlayerWhite by viewModel.playerPlayingWhite.collectAsState()
    val pieces by viewModel.pieces.collectAsState()

    val capturedPieces = remember(pieces.size) { CapturedPieces.from(pieces.map { it.toPiece() }) }

    // Top
    if (show) {
        CapturedPieceList(
            if (isPlayerWhite) capturedPieces.capturedByWhite else capturedPieces.capturedByBlack,
            if (isPlayerWhite) capturedPieces.blackScore else capturedPieces.whiteScore,
        )
    }

    content()

    // Bottom
    if (show) {
        CapturedPieceList(
            if (isPlayerWhite) capturedPieces.capturedByBlack else capturedPieces.capturedByWhite,
            if (isPlayerWhite) capturedPieces.whiteScore else capturedPieces.blackScore
        )
    }
}

@Composable
private fun CapturedPieceList(pieces: List<Byte>, score: Int) {
    LazyRow(
        modifier = Modifier
            .fillMaxWidth()
            .padding(4.dp)
            .height(24.dp)
    ) {
        items(pieces) { piece ->
            val id = when (piece) {
                Piece.PAWN -> R.drawable.ic_pawn
                Piece.KNIGHT -> R.drawable.ic_knight
                Piece.BISHOP -> R.drawable.ic_bishop
                Piece.ROOK -> R.drawable.ic_rook
                Piece.QUEEN -> R.drawable.ic_queen
                Piece.KING -> R.drawable.ic_king
                else -> throw IllegalStateException("Unknown Piece")
            }

            Icon(
                painter = painterResource(id = id),
                contentDescription = null
            )
        }

        if (score != 0) {
            item { Text(text = "+$score") }
        }
    }
}
