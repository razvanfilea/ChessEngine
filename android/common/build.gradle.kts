plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlinAndroid)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "net.theluckycoder.chess.common"
    compileSdk = Versions.Sdk.compile
    ndkVersion = "27.2.12479018"

    defaultConfig {
        minSdk = Versions.Sdk.min
        resourceConfigurations += listOf("en")

        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildFeatures.compose = true

    buildTypes {
        release {
            externalNativeBuild {
                cmake {
                    arguments += listOf("-DCMAKE_BUILD_TYPE=Release")
                }
            }
        }
    }

    externalNativeBuild {
        cmake {
            version = "3.31.1"
            path("CMakeLists.txt")
        }
    }
}

dependencies {
    debugApi(libs.kotlin.reflect)
    api(libs.kotlin.coroutinesAndroid)

    // AndroidX
    api(libs.androidx.core)
    api(libs.datastorePreferences)
    api(libs.androidx.viewmodelCompose)

    // Compose
    api(libs.compose.compiler)
    api(libs.compose.ui)
    api(libs.compose.toolingPreview)
    debugApi(libs.compose.tooling)
    api(libs.compose.foundation)
    implementation(libs.compose.material3)
    api(libs.compose.animation)
    api(libs.compose.activity)
}
