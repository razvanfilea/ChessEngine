package net.theluckycoder.chess.common

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
import net.theluckycoder.chess.common.model.SearchOptions
import kotlin.time.Duration.Companion.seconds
import kotlin.time.ExperimentalTime

private val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(
    name = "settings",
)

@OptIn(ExperimentalTime::class)
class SettingsDataStore private constructor(private val application: Application) {

    fun dataStore() = application.settingsDataStore

    fun firstStart(): Flow<Boolean> =
        dataStore().data.map { it[FIRST_START] ?: true }

    suspend fun setFirstStart(value: Boolean) = dataStore().edit { preferences ->
        preferences[FIRST_START] = value
    }

    fun showCoordinates(): Flow<Boolean> =
        dataStore().data.map { it[SHOW_COORDINATES] ?: DEFAULT_SHOW_COORDINATES }

    suspend fun setShowCoordinates(value: Boolean) =
        dataStore().edit { it[SHOW_COORDINATES] = value }

    fun showMoveHistory(): Flow<Boolean> =
        dataStore().data.map { it[SHOW_MOVES_HISTORY] ?: DEFAULT_MOVES_HISTORY }

    fun showCapturedPieces(): Flow<Boolean> =
        dataStore().data.map { it[SHOW_CAPTURED_PIECES] ?: DEFAULT_SHOW_CAPTURED_PIECES }

    fun showPieceDestination(): Flow<Boolean> =
        dataStore().data.map { it[PIECE_DESTINATIONS] ?: DEFAULT_PIECE_DESTINATIONS }

    fun centerBoard(): Flow<Boolean> =
        dataStore().data.map { it[CENTER_BOARD] ?: DEFAULT_CENTER_BOARD }

    suspend fun setDifficultyLevel(value: Int) = dataStore().edit { preferences ->
        preferences[SEARCH_DEPTH] = if (value == 0 || value == 1) value + 2 else value + 3
        preferences[QUIET_SEARCH] = value != 0
    }

    fun getEngineSettings(): Flow<SearchOptions> =
        dataStore().data.map {
            SearchOptions(
                searchDepth = it[SEARCH_DEPTH] ?: DEFAULT_SEARCH_DEPTH,
                quietSearch = it[QUIET_SEARCH] ?: DEFAULT_QUIET_SEARCH,
                searchTime = (it[SEARCH_TIME] ?: DEFAULT_SEARCH_TIME).seconds,
                threadCount = it[THREADS] ?: DEFAULT_THREADS,
                hashSize = it[HASH_SIZE] ?: DEFAULT_HASH_SIZE,
            )
        }

    suspend fun setEngineSettings(searchOptions: SearchOptions) =
        dataStore().edit { preferences ->
            preferences[SEARCH_DEPTH] = searchOptions.searchDepth
            preferences[QUIET_SEARCH] = searchOptions.quietSearch
            preferences[SEARCH_TIME] = searchOptions.searchTime.inWholeSeconds.toInt()
            preferences[THREADS] = searchOptions.threadCount
            preferences[HASH_SIZE] = searchOptions.hashSize
        }

    fun allowBook(): Flow<Boolean> =
        dataStore().data.map { it[ALLOW_BOOK] ?: true }

    fun showBasicDebug(): Flow<Boolean> =
        dataStore().data.map { it[SHOW_DEBUG_BASIC] ?: false }

    fun showAdvancedDebug(): Flow<Boolean> =
        dataStore().data.map { it[SHOW_DEBUG_ADVANCED] ?: false }

    companion object {
        @Volatile
        private var instance: SettingsDataStore? = null

        fun get(application: Application): SettingsDataStore {
            instance?.let { return it }
            synchronized(this) {
                val local = instance ?: SettingsDataStore(application)
                instance = local
                return local
            }
        }

        val FIRST_START = booleanPreferencesKey("first_start")

        val SHOW_COORDINATES = booleanPreferencesKey("show_coordinates")
        val SHOW_MOVES_HISTORY = booleanPreferencesKey("show_moves_history")
        val SHOW_CAPTURED_PIECES = booleanPreferencesKey("show_captured_pieces")
        val PIECE_DESTINATIONS = booleanPreferencesKey("piece_destinations")
        val CENTER_BOARD = booleanPreferencesKey("center_board")

        val SEARCH_DEPTH = intPreferencesKey("search_depth")
        val QUIET_SEARCH = booleanPreferencesKey("quiet_search")
        val SEARCH_TIME = intPreferencesKey("search_time")
        val THREADS = intPreferencesKey("threads")
        val HASH_SIZE = intPreferencesKey("hash_size")
        val ALLOW_BOOK = booleanPreferencesKey("allow_book")

        val SHOW_DEBUG_BASIC = booleanPreferencesKey("show_debug_basic")
        val SHOW_DEBUG_ADVANCED = booleanPreferencesKey("show_debug_advanced")

        const val DEFAULT_SHOW_COORDINATES = true
        const val DEFAULT_MOVES_HISTORY = true
        const val DEFAULT_SHOW_CAPTURED_PIECES = true
        const val DEFAULT_PIECE_DESTINATIONS = true
        const val DEFAULT_CENTER_BOARD = false

        // These will be overridden by the default [SearchOptions] in the Native Code
        const val DEFAULT_SEARCH_DEPTH = 1
        const val DEFAULT_QUIET_SEARCH = true
        const val DEFAULT_SEARCH_TIME = 30
        const val DEFAULT_THREADS = 1
        const val DEFAULT_HASH_SIZE = 64
    }
}
