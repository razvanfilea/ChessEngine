#![allow(non_snake_case)]

use std::sync::{LazyLock, Mutex, MutexGuard};

use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::{JClass, JIntArray, JObject, JString, JValue},
    sys::{JNI_VERSION_1_6, jboolean, jbyte, jint, jintArray, jlong, jobject, jstring},
};

mod engine;
mod manager;

use engine::SearchEngine;
use manager::{BoardSnapshot, ChessGame};

static GAME: LazyLock<Mutex<ChessGame>> = LazyLock::new(|| Mutex::new(ChessGame::new(true)));
static ENGINE: LazyLock<SearchEngine> = LazyLock::new(|| SearchEngine::new(64));

fn game() -> MutexGuard<'static, ChessGame> {
    GAME.lock().unwrap()
}

#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(
    _vm: *mut std::ffi::c_void,
    _reserved: *mut std::ffi::c_void,
) -> jint {
    JNI_VERSION_1_6
}

fn snapshot_to_java<'a>(
    env: &mut jni::Env<'a>,
    snapshot: &BoardSnapshot,
) -> jni::errors::Result<JObject<'a>> {
    let cls = env.find_class(jni::jni_str!(
        "net/theluckycoder/chess/common/model/BoardState"
    ))?;
    let uci_str = env.new_string(&snapshot.uci_info)?;

    let pieces_arr = env.new_int_array(snapshot.pieces.len())?;
    pieces_arr.set_region(env, 0, &snapshot.pieces)?;

    let history_arr = env.new_int_array(snapshot.moves_history.len())?;
    history_arr.set_region(env, 0, &snapshot.moves_history)?;

    env.new_object(
        cls,
        jni::jni_sig!("(IIJLjava/lang/String;ZI[I[I)V"),
        &[
            JValue::Int(snapshot.game_state),
            JValue::Int(snapshot.eval_score),
            JValue::Long(snapshot.search_time_ms),
            (&uci_str).into(),
            JValue::Bool(snapshot.is_white_turn),
            JValue::Int(snapshot.current_move_index),
            (&pieces_arr).into(),
            (&history_arr).into(),
        ],
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_initBoard(
    mut env: EnvUnowned,
    _class: JClass,
    is_player_white: jboolean,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let mut g = game();
        *g = ChessGame::new(is_player_white);
        let snap = g.get_snapshot();
        let obj = snapshot_to_java(env, &snap)?;
        Ok(obj.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_loadFenMoves(
    mut env: EnvUnowned,
    _class: JClass,
    fen: JString,
    moves: JIntArray,
    is_player_white: jboolean,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let fen_str: String = fen.mutf8_chars(env)?.to_string();
        let len = moves.len(env)?;
        let mut buf = vec![0i32; len];
        moves.get_region(env, 0, &mut buf)?;
        let maybe_snap = game().load_fen_moves(&fen_str, &buf, is_player_white);
        match maybe_snap {
            Some(snap) => {
                let obj = snapshot_to_java(env, &snap)?;
                Ok(obj.into_raw())
            }
            None => Ok(std::ptr::null_mut()),
        }
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_makeMove(
    mut env: EnvUnowned,
    _class: JClass,
    mov: jint,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let snap = game().make_move(mov);
        let obj = snapshot_to_java(env, &snap)?;
        Ok(obj.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_undo(
    mut env: EnvUnowned,
    _class: JClass,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let maybe_snap = game().undo();
        match maybe_snap {
            Some(snap) => {
                let obj = snapshot_to_java(env, &snap)?;
                Ok(obj.into_raw())
            }
            None => Ok(std::ptr::null_mut()),
        }
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_redo(
    mut env: EnvUnowned,
    _class: JClass,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let maybe_snap = game().redo();
        match maybe_snap {
            Some(snap) => {
                let obj = snapshot_to_java(env, &snap)?;
                Ok(obj.into_raw())
            }
            None => Ok(std::ptr::null_mut()),
        }
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_getBoardState(
    mut env: EnvUnowned,
    _class: JClass,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        let snap = game().get_snapshot();
        let obj = snapshot_to_java(env, &snap)?;
        Ok(obj.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_getPossibleMoves(
    mut env: EnvUnowned,
    _class: JClass,
    square: jbyte,
) -> jintArray {
    env.with_env(|env| -> jni::errors::Result<jintArray> {
        let moves = game().get_possible_moves(square as u8);
        let arr = env.new_int_array(moves.len())?;
        arr.set_region(env, 0, &moves)?;
        Ok(arr.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_getCurrentFen(
    mut env: EnvUnowned,
    _class: JClass,
) -> jstring {
    env.with_env(|env| -> jni::errors::Result<jstring> {
        let fen = game().get_current_fen();
        let s = env.new_string(&fen)?;
        Ok(s.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_getStartFen(
    mut env: EnvUnowned,
    _class: JClass,
) -> jstring {
    env.with_env(|env| -> jni::errors::Result<jstring> {
        let fen = game().get_start_fen();
        let s = env.new_string(&fen)?;
        Ok(s.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_search(
    _env: EnvUnowned,
    _class: JClass,
    depth: jint,
    max_time_ms: jlong,
    hash_size_mb: jint,
    _thread_count: jint,
) -> jint {
    // 1. Clone board while briefly locking GAME
    let board = {
        let g = game();
        g.board().clone()
    };

    // 2. Perform search decoupled from GAME mutex
    let result = ENGINE.search(board, depth, max_time_ms, hash_size_mb);

    // 3. Update search debug stats in GAME (brief lock)
    {
        let mut g = game();
        g.record_search_stats(result.search_time_ms, result.advanced_stats);
    }

    result.best_move
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_net_theluckycoder_chess_common_cpp_Native_stopSearch(
    _env: EnvUnowned,
    _class: JClass,
) {
    ENGINE.stop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_accessible_during_search() {
        let board = game().board().clone();
        let handle = std::thread::spawn(move || {
            ENGINE.search(board, 20, 5000, 16)
        });

        std::thread::sleep(std::time::Duration::from_millis(10));

        let snap = game().get_snapshot();
        assert_eq!(snap.game_state, 0);

        ENGINE.stop();
        let result = handle.join().unwrap();
        assert_ne!(result.best_move, 0);
    }
}
