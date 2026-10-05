package cloud.razvan.chess.common.ui

import androidx.compose.animation.core.animateOffsetAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import androidx.compose.ui.unit.round
import cloud.razvan.chess.common.R
import cloud.razvan.chess.common.model.GameState
import cloud.razvan.chess.common.model.GameUiState
import cloud.razvan.chess.common.model.IndexedPiece
import cloud.razvan.chess.common.model.Move
import cloud.razvan.chess.common.model.Piece

private val PIECES_RESOURCES = intArrayOf(
    R.drawable.w_pawn, R.drawable.w_knight, R.drawable.w_bishop,
    R.drawable.w_rook, R.drawable.w_queen, R.drawable.w_king,
    R.drawable.b_pawn, R.drawable.b_knight, R.drawable.b_bishop,
    R.drawable.b_rook, R.drawable.b_queen, R.drawable.b_king
)

private val TILE_WHITE = Color(0xFFEEEED2)
private val TILE_BLACK = Color(0xFF769655)
private val TILE_POSSIBLE = Color(0xCC32BF37)
private val TILE_LAST_MOVED = Color(0x8CFBC02D)
private val KING_IN_CHECK = Color(0x92DD0000)

private val PROMOTION_PIECES = listOf(Piece.QUEEN, Piece.ROOK, Piece.KNIGHT, Piece.BISHOP)

private const val PROMOTION_FIRST_ROW = 3

@Composable
fun ChessBoard(
    modifier: Modifier = Modifier,
    state: GameUiState,
    showCoordinates: Boolean,
    showPossibleMoves: Boolean,
    onPieceClick: (square: Int) -> Unit,
    onMove: (Move) -> Unit,
) = BoxWithConstraints(modifier = modifier) {
    val tileDp = min(maxWidth, maxHeight) / 8
    val tilePx = with(LocalDensity.current) { tileDp.toPx() }
    val isPlayerWhite = state.isPlayerWhite

    val movesByDest = remember(state.possibleMoves) { state.possibleMoves.groupBy { it.to.toInt() } }
    var promotionMoves by remember(state.possibleMoves) { mutableStateOf(emptyList<Move>()) }

    val onTap by rememberUpdatedState { position: Offset ->
        if (promotionMoves.isNotEmpty()) {
            if ((position.y / tilePx).toInt() - PROMOTION_FIRST_ROW in 0..1) {
                val pieceType = PROMOTION_PIECES.getOrNull((position.x / tilePx).toInt() / 2)
                promotionMoves.firstOrNull { it.promotedPieceType == pieceType }?.let(onMove)
            }
            promotionMoves = emptyList()
            return@rememberUpdatedState
        }

        val square = getSquare(isPlayerWhite, position, tilePx) ?: return@rememberUpdatedState
        val moves = movesByDest[square]
        if (moves != null) {
            if (moves.size > 1) promotionMoves = moves else onMove(moves.first())
        } else if (state.pieces.any { it.square == square && it.isWhite == isPlayerWhite }) {
            onPieceClick(square)
        }
    }

    Box(
        modifier = Modifier
            .size(tileDp * 8)
            .pointerInput(Unit) {
                detectTapGestures { onTap(it) }
            }
    ) {
        BoardTiles(
            tileDp, tilePx, isPlayerWhite,
            movesByDest = if (showPossibleMoves) movesByDest else emptyMap(),
            highlightedSquares = listOfNotNull(
                state.selectedSquare,
                state.lastMove?.from?.toInt(),
                state.lastMove?.to?.toInt(),
            ),
        )
        BoardPieces(tileDp, tilePx, isPlayerWhite, state.pieces, state.gameState)
        if (showCoordinates) {
            BoardCoordinates(tileDp, tilePx, isPlayerWhite)
        }
        if (promotionMoves.isNotEmpty()) {
            PromotionChoice(tileDp, isPlayerWhite)
        }
    }
}

@Composable
private fun PromotionChoice(tileDp: Dp, isPlayerWhite: Boolean) {
    Box(
        Modifier
            .size(tileDp * 8)
            .background(Color.Black.copy(alpha = 0.6f))
    )
    Row(
        Modifier
            .offset(y = tileDp * PROMOTION_FIRST_ROW)
            .background(TILE_WHITE)
    ) {
        for (pieceType in PROMOTION_PIECES) {
            Image(
                painter = painterResource(pieceDrawable(pieceType, isPlayerWhite)),
                contentDescription = null,
                modifier = Modifier.size(tileDp * 2),
            )
        }
    }
}

@Composable
private fun BoardTiles(
    tileDp: Dp,
    tilePx: Float,
    isPlayerWhite: Boolean,
    movesByDest: Map<Int, List<Move>>,
    highlightedSquares: List<Int>,
) {
    val possibleCapturePath = remember(tilePx) {
        val corner = tilePx / 3f
        Path().apply {
            drawTriangle(0f, 0f, corner, 0f, 0f, corner)
            drawTriangle(tilePx, 0f, tilePx - corner, 0f, tilePx, corner)
            drawTriangle(0f, tilePx, 0f, tilePx - corner, corner, tilePx)
            drawTriangle(tilePx, tilePx, tilePx, tilePx - corner, tilePx - corner, tilePx)
        }
    }

    Canvas(modifier = Modifier.size(tileDp * 8)) {
        val tileSize = Size(tilePx, tilePx)
        val possibleMoveRadius = tilePx / 6f

        for (square in 0 until 64) {
            val offset = getBoardOffset(isPlayerWhite, square, tilePx)
            val tileColor = if (isWhiteTile(square)) TILE_WHITE else TILE_BLACK

            drawRect(tileColor, topLeft = offset, size = tileSize)

            val moves = movesByDest[square]
            if (moves != null) {
                if (moves.first().isCapture) {
                    possibleCapturePath.translate(offset)
                    drawPath(path = possibleCapturePath, color = TILE_POSSIBLE)
                    possibleCapturePath.translate(-offset)
                } else {
                    drawCircle(
                        color = TILE_POSSIBLE,
                        radius = possibleMoveRadius,
                        center = offset + Offset(tilePx / 2f, tilePx / 2f)
                    )
                }
            } else if (square in highlightedSquares) {
                drawRect(TILE_LAST_MOVED, topLeft = offset, size = tileSize)
            }
        }
    }
}

@Composable
private fun BoardPieces(
    tileDp: Dp,
    tilePx: Float,
    isPlayerWhite: Boolean,
    pieces: List<IndexedPiece>,
    gameState: GameState,
) {
    val kingInCheckBrush = remember {
        Brush.radialGradient(
            listOf(KING_IN_CHECK, KING_IN_CHECK.copy(alpha = 0.55f), Color.Transparent)
        )
    }
    val kingInCheckIsWhite = when (gameState) {
        GameState.WHITE_IN_CHECK -> true
        GameState.BLACK_IN_CHECK -> false
        else -> null
    }

    for (piece in pieces) {
        key(piece.id) {
            val offset by animateOffsetAsState(getBoardOffset(isPlayerWhite, piece.square, tilePx))
            val inCheck = piece.type == Piece.KING && piece.isWhite == kingInCheckIsWhite

            Image(
                painter = painterResource(pieceDrawable(piece.type, piece.isWhite)),
                contentDescription = null,
                modifier = Modifier
                    .offset { offset.round() }
                    .size(tileDp)
                    .then(if (inCheck) Modifier.background(kingInCheckBrush, CircleShape) else Modifier)
            )
        }
    }
}

@Composable
private fun BoardCoordinates(tileDp: Dp, tilePx: Float, isPlayerWhite: Boolean) {
    val fontSize = with(LocalDensity.current) { (tileDp / 3.5f).toSp() }

    @Composable
    fun Label(text: Char, square: Int, alignment: Alignment) = Box(
        contentAlignment = alignment,
        modifier = Modifier
            .offset { getBoardOffset(isPlayerWhite, square, tilePx).round() }
            .size(tileDp)
            .padding(horizontal = 2.dp)
    ) {
        BasicText(
            text = text.toString(),
            style = TextStyle(
                fontSize = fontSize,
                // Use the color of the opposite tile, for contrast
                color = if (isWhiteTile(square)) TILE_BLACK else TILE_WHITE,
            ),
        )
    }

    val leftFile = if (isPlayerWhite) 0 else 7
    val bottomRank = if (isPlayerWhite) 0 else 7
    for (i in 0..7) {
        Label('1' + i, square = i * 8 + leftFile, Alignment.TopStart)
        Label('A' + i, square = bottomRank * 8 + i, Alignment.BottomEnd)
    }
}

private fun pieceDrawable(pieceType: Byte, isWhite: Boolean): Int =
    PIECES_RESOURCES[pieceType + if (isWhite) 0 else 6]

private fun isWhiteTile(square: Int) = (square % 8 + square / 8) % 2 == 1

private fun getBoardOffset(isPlayerWhite: Boolean, square: Int, tilePx: Float): Offset {
    val column = if (isPlayerWhite) square % 8 else 7 - square % 8
    val row = if (isPlayerWhite) 7 - square / 8 else square / 8
    return Offset(tilePx * column, tilePx * row)
}

/** Inverse of [getBoardOffset] */
private fun getSquare(isPlayerWhite: Boolean, position: Offset, tilePx: Float): Int? {
    val column = (position.x / tilePx).toInt()
    val row = (position.y / tilePx).toInt()
    if (column !in 0..7 || row !in 0..7) return null

    val file = if (isPlayerWhite) column else 7 - column
    val rank = if (isPlayerWhite) 7 - row else row
    return rank * 8 + file
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
