package net.theluckycoder.chess.common.ui

import android.app.Application
import androidx.compose.animation.core.animateOffsetAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.theluckycoder.chess.common.R
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.GameState
import net.theluckycoder.chess.common.model.IndexedPiece
import net.theluckycoder.chess.common.model.Move
import net.theluckycoder.chess.common.model.Piece
import net.theluckycoder.chess.common.model.Tile
import kotlin.math.min

private val PIECES_RESOURCES = intArrayOf(
    R.drawable.w_pawn, R.drawable.w_knight, R.drawable.w_bishop,
    R.drawable.w_rook, R.drawable.w_queen, R.drawable.w_king,
    R.drawable.b_pawn, R.drawable.b_knight, R.drawable.b_bishop,
    R.drawable.b_rook, R.drawable.b_queen, R.drawable.b_king
)

@Composable
private fun getPieceDrawable(piece: Piece): Painter {
    val type = if (piece.isWhite) piece.type.toInt() else piece.type.toInt() + 6
    return painterResource(PIECES_RESOURCES[type - 1])
}

@Composable
fun ChessBoard(
    modifier: Modifier = Modifier,
    isPlayerWhite: Boolean,
    tiles: List<Tile>,
    pieces: List<IndexedPiece>,
    gameState: GameState,
    onPieceClick: (piece: Piece) -> Unit,
    onShowPromotion: (List<Move>) -> Unit,
) = BoxWithConstraints(
    modifier = modifier
) {
    val boardSize =
        with(LocalDensity.current) { min(constraints.maxWidth, constraints.maxHeight).toDp() }
    val tileSize = boardSize / 8f

    val application = LocalContext.current.applicationContext as Application
    val dataStore = remember { SettingsDataStore.get(application) }

    // Preferences
    val showCoordinates by dataStore.showCoordinates().collectAsState(false)
    val showPossibleMoves by dataStore.showPieceDestination().collectAsState(false)

    BoardTiles(boardSize, tileSize, isPlayerWhite, tiles, showPossibleMoves, onShowPromotion)

    BoardPieces(tileSize, isPlayerWhite, pieces, gameState, onPieceClick)

    if (showCoordinates) {
        BoardCoordinates(tileSize)
    }
}

@Composable
private fun BoardTiles(
    boardDp: Dp,
    tileDp: Dp,
    isPlayerWhite: Boolean,
    tiles: List<Tile>,
    showPossibleMoves: Boolean,
    onShowPromotion: (List<Move>) -> Unit
) {
    val currentDensity = LocalDensity.current

    val whiteTileColor = colorResource(id = R.color.tile_white)
    val blackTileColor = colorResource(id = R.color.tile_black)
    val possibleTileColor = colorResource(id = R.color.tile_possible)
    val movedTileColor = colorResource(id = R.color.tile_last_moved)

    val possibleTileCircleSize = with(currentDensity) { 8.dp.toPx() }
    val tilePx = with(LocalDensity.current) { tileDp.toPx() }
    val tileSize = Size(tilePx, tilePx)

    val possibleCapturePath = remember(tilePx) {
        Path().apply {
            drawTriangle(0f, 0f, tilePx / 3f, 0f, 0f, tilePx / 3f)
            drawTriangle(tilePx, 0f, tilePx - tilePx / 3f, 0f, tilePx, tilePx / 3f)
            drawTriangle(0f, tilePx, 0f, tilePx - tilePx / 3, tilePx / 3, tilePx)
            drawTriangle(tilePx, tilePx, tilePx, tilePx - tilePx / 3, tilePx - tilePx / 3, tilePx)
        }
    }

    Canvas(modifier = Modifier.size(boardDp)) {
        tiles.forEach { tile ->
            val isWhite = (tile.square % 8 + tile.square / 8) % 2 == 1
            val tileColor = if (isWhite) whiteTileColor else blackTileColor
            val offset = getBoardOffset(isPlayerWhite, tile.square, tilePx)

            drawRect(tileColor, topLeft = offset, size = tileSize)

            when (tile.state) {
                is Tile.State.PossibleMove -> if (showPossibleMoves) {
                    val moves = tile.state.moves
                    if (moves.first().flags.capture) {
                        possibleCapturePath.translate(offset)
                        drawPath(path = possibleCapturePath, color = possibleTileColor)
                        possibleCapturePath.translate(-offset)
                    } else {
                        val center = offset + Offset(tilePx / 2f, tilePx / 2f)
                        drawCircle(
                            color = possibleTileColor,
                            radius = possibleTileCircleSize,
                            center = center
                        )
                    }
                }
                Tile.State.Moved, Tile.State.Selected ->
                    drawRect(movedTileColor, topLeft = offset, size = tileSize)
                Tile.State.None -> Unit
            }
        }
    }

    tiles
        .filter { it.state is Tile.State.PossibleMove }
        .forEach { tile ->
            val offset = getBoardOffset(isPlayerWhite, tile.square, tilePx)
            val (x, y) = with(currentDensity) { offset.x.toDp() to offset.y.toDp() }
            val state = tile.state as Tile.State.PossibleMove

            Box(
                modifier = Modifier
                    .size(tileDp)
                    .offset(x, y)
                    .clickable(role = Role.Button) {
                        val moves = state.moves
                        when {
                            moves.size > 1 -> onShowPromotion(moves)
                            else -> Native.makeMove(moves.first())
                        }
                    }
            )
        }
}

@Composable
private fun BoardCoordinates(tileDp: Dp) {
    val whiteTileColor = colorResource(id = R.color.tile_white)
    val blackTileColor = colorResource(id = R.color.tile_black)

    val textSize = 14.sp
    for (i in 1..8) {
        val textColor = if (i % 2 == 0) whiteTileColor else blackTileColor
        Text(
            text = i.toString(),
            fontSize = textSize,
            modifier = Modifier.offset(y = tileDp * (i - 1)),
            color = textColor,
        )
    }
    // TODO
    val textSizeDp = with(LocalDensity.current) { 15.5.sp.toDp() }
    for (i in 0..7) {
        val textColor = if (i % 2 == 0) whiteTileColor else blackTileColor
        Text(
            text = ('A' + i).toString(),
            fontSize = textSize,
            modifier = Modifier.offset(tileDp * (i + 1) - textSizeDp, tileDp * 8 - textSizeDp),
            color = textColor,
        )
    }
}

@Composable
private fun BoardPieces(
    tileDp: Dp,
    isPlayerWhite: Boolean,
    pieces: List<IndexedPiece>,
    gameState: GameState,
    onPieceClick: (piece: Piece) -> Unit,
) {
    val whiteInCheck = gameState == GameState.WHITE_IN_CHECK
    val blackInCheck = gameState == GameState.BLACK_IN_CHECK

    val kingInCheckColor = colorResource(id = R.color.king_in_check)
    val kingInCheckBrush = remember {
        Brush.radialGradient(
            listOf(
                kingInCheckColor,
                kingInCheckColor.copy(alpha = 0.55f),
                Color.Transparent
            )
        )
    }
    val tilePx = with(LocalDensity.current) { tileDp.toPx() }

    for (indexedPiece in pieces) {
        key(indexedPiece.id) {
            val piece = indexedPiece.toPiece()
            if (piece.square < 64 && piece.type != Piece.NONE) {
                val offset = getBoardOffset(isPlayerWhite, piece.square, tilePx)
                val animatedOffset by animateOffsetAsState(targetValue = offset)
                val (animatedX, animatedY) = with(LocalDensity.current) { animatedOffset.x.toDp() to animatedOffset.y.toDp() }

                val backgroundModifier = if (
                    (piece.type == Piece.KING)
                    && ((whiteInCheck && piece.isWhite) || (blackInCheck && !piece.isWhite))
                )
                    Modifier.background(kingInCheckBrush, CircleShape)
                else
                    Modifier

                val modifier = Modifier
                    .size(tileDp)
                    .offset(animatedX, animatedY)
                    .then(backgroundModifier)

                if (isPlayerWhite == piece.isWhite) {
                    IconButton(
                        modifier = modifier,
                        onClick = { onPieceClick(piece) }
                    ) {
                        Image(painter = getPieceDrawable(piece = piece), contentDescription = null)
                    }
                } else {
                    Image(
                        modifier = modifier,
                        painter = getPieceDrawable(piece = piece), contentDescription = null
                    )
                }
            }
        }
    }
}

private fun invertIf(invert: Boolean, i: Int) = if (invert) 7 - i else i

private fun getBoardOffset(isPlayerWhite: Boolean, index: Int, tilePx: Float): Offset {
    val x = index % 8
    val y = index / 8

    val invertedX = invertIf(!isPlayerWhite, x)
    val invertedY = invertIf(isPlayerWhite, y)

    return Offset(tilePx * invertedX, tilePx * invertedY)
}

private fun Path.drawTriangle(
    x1: Float, y1: Float,
    x2: Float, y2: Float,
    x3: Float, y3: Float
) {
    moveTo(x1, y1)
    lineTo(x2, y2)
    lineTo(x3, y3)
    lineTo(x1, y1)
}
