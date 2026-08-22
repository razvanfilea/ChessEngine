package net.theluckycoder.chess.ui.preferences

import android.app.Activity
import android.app.Application
import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.theluckycoder.chess.R
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.browseUrl
import net.theluckycoder.chess.ui.ChessMaterialTheme

class PreferencesActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val view = ComposeView(this).apply {
            setContent {
                ChessMaterialTheme {
                    PreferencesActivityContent()
                }
            }
        }

        setContentView(view)
    }
}

@Composable
private fun PreferencesActivityContent() = Scaffold(
    topBar = { Toolbar() }
) { padding ->
    val context = LocalContext.current.applicationContext as Application

    Box(modifier = Modifier.padding(padding)) {
        PreferenceScreen(
            dataStore = SettingsDataStore.get(context).dataStore(),
            items = getPreferenceItems(context),
        )
    }
}

@Composable
private fun getPreferenceItems(
    context: Context
) = listOf(
    PreferenceGroupItem(
        title = stringResource(id = R.string.pref_category_appearance),
        items = listOf(
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_coordinates),
                summary = stringResource(id = R.string.pref_coordinates_desc),
                prefKey = SettingsDataStore.SHOW_COORDINATES,
                icon = painterResource(id = R.drawable.ic_pref_coords),
                defaultValue = SettingsDataStore.DEFAULT_SHOW_COORDINATES,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_moves_history),
                summary = stringResource(id = R.string.pref_moves_history_desc),
                prefKey = SettingsDataStore.SHOW_MOVES_HISTORY,
                icon = painterResource(id = R.drawable.ic_pref_moves_history),
                defaultValue = SettingsDataStore.DEFAULT_SHOW_COORDINATES,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_captured_pieces),
                summary = stringResource(id = R.string.pref_captured_pieces_desc),
                prefKey = SettingsDataStore.SHOW_CAPTURED_PIECES,
                icon = painterResource(id = R.drawable.ic_pawn),
                defaultValue = SettingsDataStore.DEFAULT_SHOW_CAPTURED_PIECES,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_piece_destinations),
                summary = stringResource(id = R.string.pref_piece_destinations_desc),
                prefKey = SettingsDataStore.PIECE_DESTINATIONS,
                icon = painterResource(id = R.drawable.ic_pref_piece_destinations),
                defaultValue = SettingsDataStore.DEFAULT_PIECE_DESTINATIONS,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_center_board),
                summary = stringResource(id = R.string.pref_center_board_desc),
                prefKey = SettingsDataStore.CENTER_BOARD,
                icon = painterResource(id = R.drawable.ic_pref_center_board),
                defaultValue = SettingsDataStore.DEFAULT_CENTER_BOARD,
            ),
        ),
    ),

    PreferenceGroupItem(
        title = stringResource(id = R.string.pref_category_difficulty),
        items = listOf(
            SeekbarIntPreferenceItem(
                title = stringResource(id = R.string.pref_search_depth),
                summary = stringResource(id = R.string.pref_search_depth_desc),
                prefKey = SettingsDataStore.SEARCH_DEPTH,
                icon = painterResource(id = R.drawable.ic_pref_search),
                defaultValue = SettingsDataStore.DEFAULT_SEARCH_DEPTH,
                valueRange = 2..13,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_quiet_search),
                summary = stringResource(id = R.string.pref_quiet_search_desc),
                prefKey = SettingsDataStore.QUIET_SEARCH,
                icon = painterResource(id = R.drawable.ic_pref_quiet_search),
                defaultValue = SettingsDataStore.DEFAULT_QUIET_SEARCH,
            ),
            SeekbarIntPreferenceItem(
                title = stringResource(id = R.string.pref_search_time),
                summary = stringResource(id = R.string.pref_search_time_desc),
                prefKey = SettingsDataStore.SEARCH_TIME,
                icon = painterResource(id = R.drawable.ic_pref_search_time),
                defaultValue = SettingsDataStore.DEFAULT_SEARCH_TIME,
                valueRange = 1..60,
            ),
        ),
    ),

    PreferenceGroupItem(
        title = stringResource(id = R.string.pref_category_other),
        items = listOf(
            SeekbarIntPreferenceItem(
                title = stringResource(id = R.string.pref_thread_count),
                summary = stringResource(id = R.string.pref_thread_count_desc),
                prefKey = SettingsDataStore.THREADS,
                icon = painterResource(id = R.drawable.ic_pref_thread_count),
                valueRange = 1..Runtime.getRuntime().availableProcessors(),
                defaultValue = SettingsDataStore.DEFAULT_THREADS
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_allow_opening_book),
                summary = stringResource(id = R.string.pref_allow_opening_book_desc),
                prefKey = SettingsDataStore.ALLOW_BOOK,
                icon = painterResource(id = R.drawable.ic_pref_book),
                defaultValue = true
            ),
            SeekbarIntPreferenceItem(
                title = stringResource(id = R.string.pref_cache_size),
                summary = stringResource(id = R.string.pref_cache_size_desc),
                prefKey = SettingsDataStore.HASH_SIZE,
                icon = painterResource(id = R.drawable.ic_pref_cache),
                valueRange = 32..512,
                defaultValue = SettingsDataStore.DEFAULT_HASH_SIZE
            ),
        ),
    ),

    PreferenceGroupItem(
        title = stringResource(id = R.string.pref_category_about),
        items = listOf(
            EmptyPreferenceItem(
                title = stringResource(id = R.string.about_author),
                summary = "TheLuckyCoder - Filea Răzvan Gheorghe",
                icon = painterResource(id = R.drawable.ic_pref_author),
                onClick = { context.browseUrl("https://theluckycoder.net") },
            ),
            EmptyPreferenceItem(
                title = stringResource(id = R.string.about_source_code),
                summary = "Licensed under the GNU General Public License",
                icon = painterResource(id = R.drawable.ic_pref_source_code),
                onClick = { context.browseUrl("https://github.com/TheLuckyCoder/ChessEngine") }
            ),
        ),
    ),

    PreferenceGroupItem(
        title = stringResource(id = R.string.pref_category_debug),
        items = listOf(
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_debug_basic),
                summary = stringResource(id = R.string.pref_debug_basic_desc),
                prefKey = SettingsDataStore.SHOW_DEBUG_BASIC,
                icon = painterResource(id = R.drawable.ic_pref_debug_info),
                defaultValue = false,
            ),
            SwitchPreferenceItem(
                title = stringResource(id = R.string.pref_debug_advanced),
                summary = stringResource(id = R.string.pref_debug_advanced_desc),
                prefKey = SettingsDataStore.SHOW_DEBUG_ADVANCED,
                dependencyKey = SettingsDataStore.SHOW_DEBUG_BASIC,
                icon = painterResource(id = R.drawable.ic_pref_stats),
                defaultValue = false,
            )
        ),
    ),
)

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun Toolbar() {
    TopAppBar(
        title = {
            Text(stringResource(id = R.string.title_settings))
        },
        navigationIcon = {
            val activity = LocalContext.current as Activity
            IconButton(onClick = { activity.finish() }) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_arrow_back),
                    contentDescription = null
                )
            }
        }
    )
}
