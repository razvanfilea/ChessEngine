package net.theluckycoder.chess.common

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import androidx.compose.foundation.lazy.LazyListState
import androidx.core.net.toUri

fun Context.browseUrl(url: String): Boolean {
    return try {
        val intent = Intent(Intent.ACTION_VIEW).apply {
            data = url.toUri()
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }

        startActivity(intent)
        true
    } catch (e: ActivityNotFoundException) {
        e.printStackTrace()
        false
    }
}

fun LazyListState.isIndexVisible(index: Int): Boolean {
    val end = (layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: layoutInfo.totalItemsCount) - 1
    return index in firstVisibleItemIndex..end
}
