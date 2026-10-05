package cloud.razvan.chess.wearos

import cloud.razvan.chess.common.ChessApplication
import cloud.razvan.chess.common.model.Settings

class WearChessApp : ChessApplication() {

    override val defaultSettings = Settings(hashSizeMb = 16, showCoordinates = false)
}
