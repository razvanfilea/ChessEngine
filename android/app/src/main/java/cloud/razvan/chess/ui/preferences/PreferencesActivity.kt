package cloud.razvan.chess.ui.preferences

import android.app.Activity
import android.app.Application
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.LocalActivity
import androidx.activity.compose.setContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import kotlinx.coroutines.launch
import cloud.razvan.chess.R
import androidx.datastore.preferences.core.Preferences
import cloud.razvan.chess.common.SettingsDataStore
import cloud.razvan.chess.common.browseUrl
import cloud.razvan.chess.ui.ChessMaterialTheme

class PreferencesActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            ChessMaterialTheme {
                PreferencesScreen()
            }
        }
    }
}

@Composable
private fun PreferencesScreen() = Scaffold(
    topBar = { Toolbar() }
) { padding ->
    val context = LocalContext.current
    val dataStore = remember { SettingsDataStore.get(context.applicationContext as Application) }
    val scope = rememberCoroutineScope()
    val settings = dataStore.settings.collectAsState(initial = null).value ?: return@Scaffold

    fun <T> set(key: Preferences.Key<T>, value: T) {
        scope.launch { dataStore.set(key, value) }
    }

    Column(
        modifier = Modifier
            .padding(padding)
            .verticalScroll(rememberScrollState())
    ) {
        PreferenceCategory(R.string.pref_category_appearance)
        SwitchPreference(
            title = R.string.pref_coordinates,
            summary = R.string.pref_coordinates_desc,
            icon = R.drawable.ic_pref_coords,
            checked = settings.showCoordinates,
            onCheckedChange = { set(SettingsDataStore.SHOW_COORDINATES, it) },
        )
        SwitchPreference(
            title = R.string.pref_moves_history,
            summary = R.string.pref_moves_history_desc,
            icon = R.drawable.ic_pref_moves_history,
            checked = settings.showMovesHistory,
            onCheckedChange = { set(SettingsDataStore.SHOW_MOVES_HISTORY, it) },
        )
        SwitchPreference(
            title = R.string.pref_captured_pieces,
            summary = R.string.pref_captured_pieces_desc,
            icon = R.drawable.ic_pawn,
            checked = settings.showCapturedPieces,
            onCheckedChange = { set(SettingsDataStore.SHOW_CAPTURED_PIECES, it) },
        )
        SwitchPreference(
            title = R.string.pref_piece_destinations,
            summary = R.string.pref_piece_destinations_desc,
            icon = R.drawable.ic_pref_piece_destinations,
            checked = settings.showPossibleMoves,
            onCheckedChange = { set(SettingsDataStore.PIECE_DESTINATIONS, it) },
        )

        PreferenceCategory(R.string.pref_category_difficulty)
        SliderPreference(
            title = R.string.pref_difficulty_level,
            summary = R.string.pref_difficulty_level_desc,
            icon = R.drawable.ic_pref_search,
            value = settings.difficultyLevel,
            range = SettingsDataStore.DIFFICULTY_LEVELS,
            onValueChange = { set(SettingsDataStore.DIFFICULTY_LEVEL, it) },
        )
        SliderPreference(
            title = R.string.pref_search_time,
            summary = R.string.pref_search_time_desc,
            icon = R.drawable.ic_pref_search_time,
            value = settings.searchTimeSeconds,
            range = 1..60,
            onValueChange = { set(SettingsDataStore.SEARCH_TIME, it) },
        )

        PreferenceCategory(R.string.pref_category_other)
        SliderPreference(
            title = R.string.pref_thread_count,
            summary = R.string.pref_thread_count_desc,
            icon = R.drawable.ic_pref_thread_count,
            value = settings.threads,
            range = 1..Runtime.getRuntime().availableProcessors(),
            onValueChange = { set(SettingsDataStore.THREADS, it) },
        )
        SliderPreference(
            title = R.string.pref_cache_size,
            summary = R.string.pref_cache_size_desc,
            icon = R.drawable.ic_pref_cache,
            value = settings.hashSizeMb,
            range = SettingsDataStore.HASH_SIZE_RANGE,
            onValueChange = { set(SettingsDataStore.HASH_SIZE, it) },
        )

        PreferenceCategory(R.string.pref_category_about)
        Preference(
            title = stringResource(R.string.about_author),
            summary = "Filea Răzvan Gheorghe",
            icon = R.drawable.ic_pref_author,
            onClick = { context.browseUrl("https://github.com/razvanfilea") },
        )
        Preference(
            title = stringResource(R.string.about_source_code),
            summary = "Licensed under the GNU General Public License",
            icon = R.drawable.ic_pref_source_code,
            onClick = { context.browseUrl("https://github.com/razvanfilea/ChessEngine") },
        )

        PreferenceCategory(R.string.pref_category_debug)
        val debugBasic = settings.showDebugBasic
        SwitchPreference(
            title = R.string.pref_debug_basic,
            summary = R.string.pref_debug_basic_desc,
            icon = R.drawable.ic_pref_debug_info,
            checked = debugBasic,
            onCheckedChange = { set(SettingsDataStore.SHOW_DEBUG_BASIC, it) },
        )
        AnimatedVisibility(debugBasic) {
            SwitchPreference(
                title = R.string.pref_debug_advanced,
                summary = R.string.pref_debug_advanced_desc,
                icon = R.drawable.ic_pref_stats,
                checked = settings.showDebugAdvanced,
                onCheckedChange = { set(SettingsDataStore.SHOW_DEBUG_ADVANCED, it) },
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun Toolbar() = TopAppBar(
    title = { Text(stringResource(R.string.title_settings)) },
    navigationIcon = {
        val activity = LocalActivity.current as Activity
        IconButton(onClick = { activity.finish() }) {
            Icon(
                painter = painterResource(R.drawable.ic_arrow_back),
                contentDescription = null
            )
        }
    }
)
