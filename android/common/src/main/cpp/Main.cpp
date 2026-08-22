#include <jni.h>

#include <iostream>

#include "JniUtils.h"
#include "AndroidBuffer.h"

#include "chess/BoardManager.h"
#include "chess/Stats.h"
#include "chess/Tests.h"
#include "chess/algorithm/Evaluation.h"
#include "chess/MoveGen.h"
#include "chess/algorithm/Search.h"
#include "chess/polyglot/PolyBook.h"

#define ExportFunction extern "C" JNIEXPORT

static constexpr auto TAG = "ChessCpp";
static JavaVM *jvm = nullptr;
static jobject boardChangeListener = nullptr;
static jobject searchListener = nullptr;

const BoardManager::BoardChangedCallback boardChangedCallback = [](const GameState state)
{
	NativeThreadAttach threadAttach{ jvm };
	auto env = threadAttach.getEnv();

	const static auto callbackId =
		env->GetMethodID(JniCache::boardChangeListenerClass, "boardChanged", "(I)V");
	if (boardChangeListener == nullptr)
		LOGE(TAG, "No BoardChangeListener Instance found");
	else
		env->CallVoidMethod(boardChangeListener, callbackId, static_cast<jint>(state));
};

jobject nativeMoveToJObject(JNIEnv *pEnv, Move move)
{
	const static auto constructorId = pEnv->GetMethodID(JniCache::moveClass, "<init>", "(IBBBBBB)V");

	return pEnv->NewObject(JniCache::moveClass, constructorId,
							   static_cast<jint>(move.getContents()),
							   static_cast<jbyte>(move.from()),
							   static_cast<jbyte>(move.to()),
							   static_cast<jbyte>(move.piece()),
							   static_cast<jbyte>(move.capturedPiece()),
							   static_cast<jbyte>(move.promotedPiece()),
							   static_cast<jbyte>(move.flags().getContents()));
}

const BoardManager::SearchFinishedCallback searchFinishedCallback = [](const bool success)
{
	NativeThreadAttach threadAttach{ jvm };
	auto env = threadAttach.getEnv();

	const static auto callbackId =
		env->GetMethodID(JniCache::searchListenerClass, "onFinish", "(Z)V");
	if (searchListener == nullptr)
		LOGE(TAG, "No SearchListener Instance found");
	else
		env->CallVoidMethod(searchListener, callbackId, success);
};

ExportFunction jint JNI_OnLoad(JavaVM *vm, void *)
{
	LOGI(TAG, "JNI_OnLoad");

	JNIEnv *env;
	if (vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6) != JNI_OK)
		return -1;

	jvm = vm;

	std::cout.rdbuf(new AndroidBuffer);

	JniCache::createCaches(env);

	return JNI_VERSION_1_6;
}

ExportFunction void JNI_OnUnload(JavaVM *vm, void *)
{
	LOGI(TAG, "JNI_OnUnload");

	JNIEnv *env;
	vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6);

	// Clean the caches
	env->DeleteGlobalRef(boardChangeListener);
	env->DeleteGlobalRef(searchListener);
	boardChangeListener = nullptr;
	searchListener = nullptr;
	JniCache::cleanCaches(env);

	delete std::cout.rdbuf(nullptr);

	jvm = nullptr;
}

// region Helper Functions

// endregion

// Native Class

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_initBoard(JNIEnv *pEnv, jobject, jobject instance,
														 jboolean isPlayerWhite)
{
	pEnv->ExceptionClear();

	if (!pEnv->IsSameObject(boardChangeListener, instance))
	{
		LOGD(TAG, "initBoardNative");
		if (boardChangeListener)
			pEnv->DeleteGlobalRef(boardChangeListener);

		boardChangeListener = pEnv->NewGlobalRef(instance);

		BoardManager::initBoardManager(boardChangedCallback);
	}

	BoardManager::initBoardManager(boardChangedCallback, isPlayerWhite);
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_initBook(JNIEnv *pEnv, jobject, jstring bookPath)
{
	const char *nativeString = pEnv->GetStringUTFChars(bookPath, nullptr);
	PolyBook::initBook(nativeString);
	LOGV("Book", "Initialized");

	pEnv->ReleaseStringUTFChars(bookPath, nativeString);
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_enableBook(JNIEnv *, jobject, jboolean enable)
{
	PolyBook::enable(static_cast<bool>(enable));
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_setSearchListener(JNIEnv *pEnv, jobject, jobject instance)
{
	if (!pEnv->IsSameObject(searchListener, instance))
	{
		if (boardChangeListener)
			pEnv->DeleteGlobalRef(searchListener);
		searchListener = pEnv->NewGlobalRef(instance);
	}
	BoardManager::setSearchFinishedCallback(searchFinishedCallback);
}

ExportFunction jboolean JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_isEngineBusy(JNIEnv *, jobject)
{
	return static_cast<jboolean>(BoardManager::isEngineBusy());
}

ExportFunction jboolean JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_isPlayerWhite(JNIEnv *, jobject)
{
	return static_cast<jboolean>(BoardManager::isPlayerWhite());
}

ExportFunction jboolean JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_isPlayersTurn(JNIEnv *, jobject)
{
	return static_cast<jboolean>(BoardManager::isPlayerWhite() == BoardManager::getBoard().colorToMove);
}

ExportFunction jobjectArray JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getPieces(JNIEnv *pEnv, jobject)
{
	pEnv->ExceptionClear();

	const static auto constructorId = pEnv->GetMethodID(JniCache::indexedPieceClass, "<init>", "(IIBZ)V");

	auto &&pieceList = BoardManager::getMovesStack().getIndexedPieces();

	jobjectArray array =
		pEnv->NewObjectArray(static_cast<jsize>(pieceList.size()), JniCache::indexedPieceClass, nullptr);

	for (size_t i{}; i < pieceList.size(); ++i)
	{
		const auto &it = pieceList[i];
		jobject obj = pEnv->NewObject(JniCache::indexedPieceClass, constructorId, it.id, it.square, it.pieceType,
									  it.pieceColor);
		pEnv->SetObjectArrayElement(array, i, obj);
	}

	return array;
}

ExportFunction jobjectArray JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getPossibleMoves(JNIEnv *pEnv, jobject, jbyte square)
{
	const auto possibleMoves = BoardManager::getPossibleMoves(toSquare(u8(square)));

	jobjectArray result =
		pEnv->NewObjectArray(static_cast<jsize>(possibleMoves.size()), JniCache::moveClass, nullptr);

	usize i{};
	for (auto &&move : possibleMoves)
	{
		jobject obj = nativeMoveToJObject(pEnv, move);
		pEnv->SetObjectArrayElement(result, i++, obj);
	}

	return result;
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_makeMove(JNIEnv *, jobject, jint move)
{
	BoardManager::makeMove(Move{ static_cast<u32>(move) });
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_makeEngineMove(JNIEnv *, jobject)
{
	BoardManager::makeEngineMove();
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_stopSearch(JNIEnv *, jobject)
{
	Search::stopSearch();
}


ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_undoMoves(JNIEnv *, jobject)
{
	BoardManager::undoLastMoves();
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_redoMoves(JNIEnv *, jobject)
{
	BoardManager::redoLastMoves();
}

ExportFunction jboolean JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_loadFen(JNIEnv *pEnv, jobject, jboolean playerWhite,
													   jstring fenPosition)
{
	const char *nativeString = pEnv->GetStringUTFChars(fenPosition, nullptr);

	const bool loaded = BoardManager::loadGame(playerWhite, nativeString);

	pEnv->ReleaseStringUTFChars(fenPosition, nativeString);

	return static_cast<jboolean>(loaded);
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_loadFenMoves(JNIEnv *pEnv, jobject,
															jboolean isPlayerWhite, jstring fenPosition,
															jintArray moves)
{
	const usize size = pEnv->GetArrayLength(moves);
	auto elements = pEnv->GetIntArrayElements(moves, JNI_FALSE);

	std::vector<Move> vector;
	vector.reserve(size);

	for (usize i = 0; i < size; ++i)
		vector.emplace_back(elements[i]);

	const char *nativeString = pEnv->GetStringUTFChars(fenPosition, nullptr);

	BoardManager::loadGame(isPlayerWhite, nativeString, vector);

	pEnv->ReleaseStringUTFChars(fenPosition, nativeString);
}

ExportFunction jstring JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getCurrentFen(JNIEnv *pEnv, jobject)
{
	const auto fenString = BoardManager::getBoard().getFen();
	return pEnv->NewStringUTF(fenString.c_str());
}

ExportFunction jstring JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getStartFen(JNIEnv *pEnv, jobject)
{
	const auto &fenString = BoardManager::getMovesStack().getStartFen();
	return pEnv->NewStringUTF(fenString.c_str());
}

ExportFunction jobjectArray JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getMovesHistory(JNIEnv *pEnv, jobject)
{
	const auto stack = BoardManager::getMovesStack();

	const auto result =
		pEnv->NewObjectArray(static_cast<jsize>(stack.size()), JniCache::moveClass, nullptr);

	usize i{};
	for (auto &&board : stack)
	{
		jobject obj = nativeMoveToJObject(pEnv, board.getMove());
		pEnv->SetObjectArrayElement(result, i++, obj);
	}

	return result;
}

ExportFunction jint JNICALL
Java_net_theluckycoder_chess_common_cpp_Native_getCurrentMoveIndex(JNIEnv *, jobject)
{
	return BoardManager::getMovesStack().getCurrentIndex();
}

// region Tests

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_cpp_Tests_perftTests(JNIEnv *, jobject)
{
	Tests::runPerftTests();
}

ExportFunction jstring JNICALL
Java_net_theluckycoder_chess_common_cpp_Tests_evaluationTests(JNIEnv *pEnv, jobject)
{
	const auto testResults = Tests::runEvaluationTests();
	if (testResults.empty())
		return nullptr;

	return pEnv->NewStringUTF(testResults.c_str());
}

// endregion Tests

// region SearchOptions

ExportFunction jobject JNICALL
Java_net_theluckycoder_chess_common_model_SearchOptions_getNativeSearchOptions(JNIEnv *pEnv, jclass)
{
	const static auto constructorId = pEnv->GetMethodID(JniCache::searchOptionsClass, "<init>", "(IZIJI)V");

	const auto options = BoardManager::getSearchOptions();

	return pEnv->NewObject(JniCache::searchOptionsClass, constructorId,
						   options.depth(), options.quietSearch(), options.threadCount(),
						   static_cast<jlong>(options.searchTime()), options.tableSizeMb());
}

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_model_SearchOptions_setNativeSearchOptions(JNIEnv *, jclass,
																			   jint searchDepth,
																			   jboolean quietSearch,
																			   jint threadCount,
																			   jint hashSizeMb,
																			   jlong searchTime)
{
	BoardManager::setSearchOptions({ searchDepth,
									 static_cast<u32>(threadCount),
									 static_cast<u32>(hashSizeMb),
									 static_cast<bool>(quietSearch),
									 static_cast<i64>(searchTime) });
}

// endregion SearchOptions

// region DebugStats

ExportFunction void JNICALL
Java_net_theluckycoder_chess_common_model_DebugStats_enable(JNIEnv *, jclass, jboolean enabled)
{
	Stats::setEnabled(enabled);
}

ExportFunction jlong JNICALL
Java_net_theluckycoder_chess_common_model_DebugStats_getNativeSearchTime(JNIEnv *, jclass)
{
	return static_cast<jlong>(Stats::getElapsedMs());
}

ExportFunction jint JNICALL
Java_net_theluckycoder_chess_common_model_DebugStats_getNativeBoardEvaluation(JNIEnv *, jclass)
{
	return static_cast<jint>(Evaluation::value(BoardManager::getBoard()));
}

ExportFunction jstring JNICALL
Java_net_theluckycoder_chess_common_model_DebugStats_getNativeAdvancedStats(JNIEnv *pEnv, jclass)
{
	const auto stats = Stats::formatStats('\n');
	return pEnv->NewStringUTF(stats.c_str());
}

// endregion DebugStats
