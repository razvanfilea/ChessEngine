package net.theluckycoder.chess.common.cpp

import androidx.annotation.Keep

interface BoardChangeListener {

    @Keep // Called by native code
    fun boardChanged(gameStateInt: Int)
}

