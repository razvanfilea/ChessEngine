package cloud.razvan.chess.common

import android.app.Application
import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.intPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import cloud.razvan.chess.common.model.Settings

private val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(
    name = "settings",
)

class SettingsDataStore private constructor(private val application: Application) {

    /** Used for every setting the user hasn't changed */
    val defaults: Settings = (application as ChessApplication).defaultSettings

    val settings: Flow<Settings> = application.settingsDataStore.data.map { prefs ->
        Settings(
            showCoordinates = prefs[SHOW_COORDINATES] ?: defaults.showCoordinates,
            showMovesHistory = prefs[SHOW_MOVES_HISTORY] ?: defaults.showMovesHistory,
            showCapturedPieces = prefs[SHOW_CAPTURED_PIECES] ?: defaults.showCapturedPieces,
            showPossibleMoves = prefs[PIECE_DESTINATIONS] ?: defaults.showPossibleMoves,
            difficultyLevel = (prefs[DIFFICULTY_LEVEL] ?: defaults.difficultyLevel).coerceIn(DIFFICULTY_LEVELS),
            searchTimeSeconds = prefs[SEARCH_TIME] ?: defaults.searchTimeSeconds,
            threads = prefs[THREADS] ?: defaults.threads,
            hashSizeMb = (prefs[HASH_SIZE] ?: defaults.hashSizeMb).coerceIn(HASH_SIZE_RANGE),
            showDebugBasic = prefs[SHOW_DEBUG_BASIC] ?: defaults.showDebugBasic,
            showDebugAdvanced = prefs[SHOW_DEBUG_ADVANCED] ?: defaults.showDebugAdvanced,
        )
    }

    suspend fun <T> set(key: Preferences.Key<T>, value: T) {
        application.settingsDataStore.edit { it[key] = value }
    }

    companion object {
        private var instance: SettingsDataStore? = null

        fun get(application: Application): SettingsDataStore =
            instance ?: SettingsDataStore(application).also { instance = it }

        val SHOW_COORDINATES = booleanPreferencesKey("show_coordinates")
        val SHOW_MOVES_HISTORY = booleanPreferencesKey("show_moves_history")
        val SHOW_CAPTURED_PIECES = booleanPreferencesKey("show_captured_pieces")
        val PIECE_DESTINATIONS = booleanPreferencesKey("piece_destinations")

        val DIFFICULTY_LEVEL = intPreferencesKey("difficulty_level")
        val SEARCH_TIME = intPreferencesKey("search_time")
        val THREADS = intPreferencesKey("threads")
        val HASH_SIZE = intPreferencesKey("hash_size")

        val SHOW_DEBUG_BASIC = booleanPreferencesKey("show_debug_basic")
        val SHOW_DEBUG_ADVANCED = booleanPreferencesKey("show_debug_advanced")

        val DIFFICULTY_LEVELS = 1..8
        val HASH_SIZE_RANGE = 16..256
    }
}
