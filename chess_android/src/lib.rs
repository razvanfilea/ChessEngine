#![allow(non_snake_case)]

use std::sync::{LazyLock, Mutex, MutexGuard};

use chess_core::prelude::*;
use jni::{
    EnvUnowned,
    errors::ThrowRuntimeExAndDefault,
    objects::{JClass, JObject, JString, JValue},
    sys::{jboolean, jint, jlong, jobject, jstring},
};

mod engine;
mod manager;
mod notation;
mod rules;

use engine::SearchEngine;
use manager::{BoardSnapshot, ChessGame};

static GAME: LazyLock<Mutex<ChessGame>> = LazyLock::new(|| Mutex::new(ChessGame::new()));
static ENGINE: LazyLock<SearchEngine> = LazyLock::new(|| SearchEngine::new(64));

fn game() -> MutexGuard<'static, ChessGame> {
    GAME.lock().unwrap()
}

fn snapshot_to_java<'a>(
    env: &mut jni::Env<'a>,
    snapshot: &BoardSnapshot,
) -> jni::errors::Result<JObject<'a>> {
    let cls = env.find_class(jni::jni_str!("cloud/razvan/chess/common/model/BoardState"))?;
    let uci_str = env.new_string(&snapshot.uci_info)?;

    let pieces_arr = env.new_int_array(snapshot.pieces.len())?;
    pieces_arr.set_region(env, 0, &snapshot.pieces)?;

    let history_arr = env.new_int_array(snapshot.moves_history.len())?;
    history_arr.set_region(env, 0, &snapshot.moves_history)?;

    let san_arr = env.new_object_array(
        snapshot.moves_san.len() as i32,
        jni::jni_str!("java/lang/String"),
        JObject::null(),
    )?;
    for (i, san) in snapshot.moves_san.iter().enumerate() {
        let s = env.new_string(san)?;
        san_arr.set_element(env, i, &s)?;
    }

    let legal_arr = env.new_int_array(snapshot.legal_moves.len())?;
    legal_arr.set_region(env, 0, &snapshot.legal_moves)?;

    env.new_object(
        cls,
        jni::jni_sig!("(IIJLjava/lang/String;ZI[I[I[Ljava/lang/String;[I)V"),
        &[
            JValue::Int(snapshot.game_state),
            JValue::Int(snapshot.eval_score),
            JValue::Long(snapshot.search_time_ms),
            (&uci_str).into(),
            JValue::Bool(snapshot.is_white_turn),
            JValue::Int(snapshot.current_move_index),
            (&pieces_arr).into(),
            (&history_arr).into(),
            (&san_arr).into(),
            (&legal_arr).into(),
        ],
    )
}

/// Runs `f` and returns its snapshot as a `BoardState`, or null for `None`
fn return_snapshot<'local>(
    mut env: EnvUnowned<'local>,
    f: impl FnOnce(&mut jni::Env<'local>) -> jni::errors::Result<Option<BoardSnapshot>>,
) -> jobject {
    env.with_env(|env| -> jni::errors::Result<jobject> {
        match f(env)? {
            Some(snap) => Ok(snapshot_to_java(env, &snap)?.into_raw()),
            None => Ok(std::ptr::null_mut()),
        }
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

fn return_string<'local>(
    mut env: EnvUnowned<'local>,
    f: impl FnOnce(&mut jni::Env<'local>) -> jni::errors::Result<String>,
) -> jstring {
    env.with_env(|env| -> jni::errors::Result<jstring> {
        let s = f(env)?;
        Ok(env.new_string(&s)?.into_raw())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

fn replace_game(new_game: Option<ChessGame>) -> Option<BoardSnapshot> {
    let new_game = new_game?;
    ENGINE.new_game();
    let mut g = game();
    g.replace(new_game);
    Some(g.snapshot(0, String::new()))
}

fn player_color(is_player_white: bool) -> Color {
    if is_player_white {
        Color::White
    } else {
        Color::Black
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_initBoard(
    env: EnvUnowned,
    _class: JClass,
) -> jobject {
    return_snapshot(env, |_| Ok(replace_game(Some(ChessGame::new()))))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_loadFen(
    env: EnvUnowned,
    _class: JClass,
    fen: JString,
) -> jobject {
    return_snapshot(env, |env| {
        let fen: String = fen.mutf8_chars(env)?.to_string();
        Ok(replace_game(ChessGame::load(&fen, &[])))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_loadGame(
    env: EnvUnowned,
    _class: JClass,
    save: JString,
) -> jobject {
    return_snapshot(env, |env| {
        let save: String = save.mutf8_chars(env)?.to_string();
        Ok(replace_game(ChessGame::load_save(&save)))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_saveGame(
    env: EnvUnowned,
    _class: JClass,
) -> jstring {
    return_string(env, |_| Ok(game().save()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_makeMove(
    env: EnvUnowned,
    _class: JClass,
    mov: jint,
) -> jobject {
    return_snapshot(env, |_| {
        let mut g = game();
        g.make_move(mov);
        Ok(Some(g.snapshot(0, String::new())))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_undo(
    env: EnvUnowned,
    _class: JClass,
    is_player_white: jboolean,
) -> jobject {
    return_snapshot(env, |_| {
        let mut g = game();
        Ok(g.undo(player_color(is_player_white))
            .then(|| g.snapshot(0, String::new())))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_redo(
    env: EnvUnowned,
    _class: JClass,
    is_player_white: jboolean,
) -> jobject {
    return_snapshot(env, |_| {
        let mut g = game();
        Ok(g.redo(player_color(is_player_white))
            .then(|| g.snapshot(0, String::new())))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_getCurrentFen(
    env: EnvUnowned,
    _class: JClass,
) -> jstring {
    return_string(env, |_| Ok(game().get_current_fen()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_exportPgn(
    env: EnvUnowned,
    _class: JClass,
    date: JString,
    is_player_white: jboolean,
) -> jstring {
    return_string(env, |env| {
        let date: String = date.mutf8_chars(env)?.to_string();
        Ok(game().pgn(&date, is_player_white))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_engineMove(
    env: EnvUnowned,
    _class: JClass,
    depth: jint,
    time_ms: jlong,
    hash_mb: jint,
    _threads: jint,
) -> jobject {
    return_snapshot(env, |_| {
        let (board, position_keys, version, stop_count) = {
            let g = game();
            if g.game_state().is_game_over() {
                return Ok(None);
            }
            let keys = g.position_keys();
            (g.board().clone(), keys, g.version(), ENGINE.stop_count())
        };

        let Some(result) =
            ENGINE.search(board, &position_keys, depth, time_ms, hash_mb, stop_count)
        else {
            return Ok(None);
        };

        let mut g = game();
        let unchanged = g.version() == version && ENGINE.stop_count() == stop_count;
        if !unchanged || !g.make_move(result.best_move.bits() as i32) {
            return Ok(None);
        }
        Ok(Some(g.snapshot(result.search_time_ms, result.uci_info)))
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_cloud_razvan_chess_common_Native_stopSearch(
    _env: EnvUnowned,
    _class: JClass,
) {
    ENGINE.stop();
}
