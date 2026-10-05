package cloud.razvan.chess.common.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.expandIn
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkOut
import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
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
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import cloud.razvan.chess.common.CapturedPieces
import cloud.razvan.chess.common.R
import cloud.razvan.chess.common.isIndexVisible
import cloud.razvan.chess.common.model.IndexedPiece
import cloud.razvan.chess.common.model.Move
import cloud.razvan.chess.common.model.Piece
import kotlin.random.Random

enum class Side(
    @DrawableRes val painterRes: Int,
    val backgroundColor: Color,
    @StringRes val contentDescriptionRes: Int,
) {
    WHITE(R.drawable.w_king, Color(0xFFF1F1F1), R.string.side_white),
    BLACK(R.drawable.b_king, Color(0xFF1E1E1E), R.string.side_black),
    RANDOM(R.drawable.side_random, Color(0xFF757575), R.string.side_random);

    fun isPlayerWhite(): Boolean = when (this) {
        WHITE -> true
        BLACK -> false
        RANDOM -> Random.nextBoolean()
    }
}

@Composable
fun ChooseSidesToggle(
    modifier: Modifier = Modifier,
    selectedSide: MutableState<Side>,
    primaryColor: Color,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(8.dp)
            .clip(RoundedCornerShape(8.dp))
    ) {
        Side.entries.forEach { side ->
            val selected = selectedSide.value == side
            val backgroundColor = if (selected)
                primaryColor.copy(alpha = 0.5f)
            else
                side.backgroundColor

            Box(
                modifier = Modifier
                    .weight(1f)
                    .background(backgroundColor)
                    .selectable(
                        selected = selected,
                        role = Role.RadioButton,
                        onClick = { selectedSide.value = side },
                    )
                    .padding(4.dp),
                contentAlignment = Alignment.Center,
            ) {
                Image(
                    modifier = Modifier.size(54.dp),
                    painter = painterResource(id = side.painterRes),
                    contentDescription = stringResource(id = side.contentDescriptionRes),
                )
            }
        }
    }
}

@Composable
fun MovesHistory(
    show: Boolean,
    movesHistory: List<Move>,
    currentMoveIndex: Int,
    textColor: Color,
    modifier: Modifier = Modifier,
    backgroundColor: Color = Color(0xFF222222),
) {
    val transition =
        remember { MutableTransitionState(initialState = show) }.apply { targetState = show }

    AnimatedVisibility(
        visibleState = transition,
        modifier = modifier,
        enter = fadeIn() + expandIn(expandFrom = Alignment.TopCenter),
        exit = shrinkOut(shrinkTowards = Alignment.TopCenter) + fadeOut()
    ) {
        val listState = rememberLazyListState(
            initialFirstVisibleItemIndex = currentMoveIndex.coerceAtLeast(0)
        )

        LaunchedEffect(movesHistory, currentMoveIndex) {
            if (movesHistory.isNotEmpty()
                && currentMoveIndex in movesHistory.indices
                && !listState.isIndexVisible(currentMoveIndex)
            ) listState.animateScrollToItem(currentMoveIndex)
        }

        val moveStyle = TextStyle(color = textColor, fontSize = 13.sp)

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
                                BasicText(
                                    text = "${index / 2 + 1}. ",
                                    style = moveStyle.copy(color = Color.Gray),
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
                            BasicText(
                                modifier = modifier,
                                text = item.san,
                                style = moveStyle,
                            )
                        }
                    }
                } else
                    item { BasicText(modifier = Modifier.padding(1.dp), text = "", style = moveStyle) }
            }
        )
    }
}

@Composable
fun CapturedPiecesLists(
    modifier: Modifier = Modifier,
    show: Boolean,
    isPlayerWhite: Boolean,
    pieces: List<IndexedPiece>,
    contentColor: Color,
    content: @Composable ColumnScope.() -> Unit
) = Column(modifier) {
    val capturedPieces = remember(pieces) { CapturedPieces.from(pieces) }

    if (show) {
        CapturedPieceList(
            if (isPlayerWhite) capturedPieces.capturedByWhite else capturedPieces.capturedByBlack,
            if (isPlayerWhite) capturedPieces.blackScore else capturedPieces.whiteScore,
            contentColor,
        )
    }

    content()

    if (show) {
        CapturedPieceList(
            if (isPlayerWhite) capturedPieces.capturedByBlack else capturedPieces.capturedByWhite,
            if (isPlayerWhite) capturedPieces.whiteScore else capturedPieces.blackScore,
            contentColor,
        )
    }
}

@Composable
private fun CapturedPieceList(pieces: List<Byte>, score: Int, contentColor: Color) {
    LazyRow(
        modifier = Modifier
            .fillMaxWidth()
            .padding(4.dp)
            .height(24.dp),
        verticalAlignment = Alignment.CenterVertically,
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

            Image(
                modifier = Modifier.size(24.dp),
                painter = painterResource(id = id),
                colorFilter = ColorFilter.tint(contentColor),
                contentDescription = null
            )
        }

        if (score != 0) {
            item { BasicText(text = "+$score", style = TextStyle(color = contentColor, fontSize = 16.sp)) }
        }
    }
}
