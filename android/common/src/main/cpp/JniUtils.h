#pragma once

#include <android/log.h>
#include <cassert>

#define LOGV(LOG_TAG, ...) __android_log_print(ANDROID_LOG_VERBOSE, LOG_TAG, __VA_ARGS__)
#define LOGI(LOG_TAG, ...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGD(LOG_TAG, ...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGW(LOG_TAG, ...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)
#define LOGE(LOG_TAG, ...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

/**
 * This class automatically attaches this thread to the JVM if necessary,
 * and detaches it upon destruction.
 * And it provides a valid pointer to the JNIEnv.
 *
 * In this way it works similarly to the std::lock_guard class.
 */
class NativeThreadAttach
{
public:
	explicit NativeThreadAttach(JavaVM *jvm)
		: _jvm(jvm)
	{
		_jvmEnvState = _jvm->GetEnv(reinterpret_cast<void **>(&_env), JNI_VERSION_1_6);

		if (_jvmEnvState == JNI_EDETACHED)
			_jvm->AttachCurrentThread(&_env, nullptr);
		_env->ExceptionClear();
	}

	JNIEnv *getEnv() { return _env; }

	~NativeThreadAttach()
	{
		if (_jvmEnvState == JNI_EDETACHED)
			_jvm->DetachCurrentThread();
	}

private:
	JavaVM *_jvm;
	JNIEnv *_env = nullptr;
	int _jvmEnvState{};
};

namespace JniCache
{
	jclass boardChangeListenerClass;
	jclass searchListenerClass;
	jclass indexedPieceClass;
	jclass moveClass;
	jclass searchOptionsClass;

	static jclass cacheClass(JNIEnv *env, const char *name)
	{
		return static_cast<jclass>(env->NewGlobalRef(env->FindClass(name)));
	}

	void createCaches(JNIEnv *env)
	{
		boardChangeListenerClass = cacheClass(env, "net/theluckycoder/chess/common/cpp/BoardChangeListener");
		searchListenerClass = cacheClass(env, "net/theluckycoder/chess/common/cpp/SearchListener");
		indexedPieceClass = cacheClass(env, "net/theluckycoder/chess/common/model/IndexedPiece");
		moveClass = cacheClass(env, "net/theluckycoder/chess/common/model/Move");
		searchOptionsClass = cacheClass(env, "net/theluckycoder/chess/common/model/SearchOptions");
	}

	void cleanCaches(JNIEnv *env)
	{
		env->DeleteGlobalRef(boardChangeListenerClass);
		env->DeleteGlobalRef(searchListenerClass);
		env->DeleteGlobalRef(indexedPieceClass);
		env->DeleteGlobalRef(moveClass);
		env->DeleteGlobalRef(searchOptionsClass);
	}
}

