plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.linkcam.app"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.linkcam.app"
        minSdk = 29 // Android 10+
        targetSdk = 34
        versionCode = 15
        versionName = "1.2.5"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    signingConfigs {
        create("release") {
            val keystoreFile = rootProject.file("release.jks")
            if (keystoreFile.exists()) {
                storeFile = keystoreFile
                storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD") ?: "linkcam_release"
                keyAlias = System.getenv("ANDROID_KEY_ALIAS") ?: "linkcam"
                keyPassword = System.getenv("ANDROID_KEY_PASSWORD") ?: "linkcam_release"
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            isShrinkResources = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            val releaseSigning = signingConfigs.findByName("release")
            if (releaseSigning?.storeFile?.exists() == true) {
                signingConfig = releaseSigning
            } else {
                signingConfig = signingConfigs.getByName("debug")
            }
        }
        debug {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }

    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.11"
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    // Kotlin Standard Library BOM
    implementation(platform("org.jetbrains.kotlin:kotlin-bom:1.9.23"))
    implementation("org.jetbrains.kotlin:kotlin-stdlib")

    // AndroidX & Lifecycle
    implementation("androidx.core:core-ktx:1.13.1")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.7.0")
    implementation("androidx.activity:activity-compose:1.9.0")

    // Jetpack Compose & Material 3
    implementation(platform("androidx.compose:compose-bom:2024.05.00"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")

    // CameraX v1.3+
    val cameraxVersion = "1.3.3"
    implementation("androidx.camera:camera-core:$cameraxVersion")
    implementation("androidx.camera:camera-camera2:$cameraxVersion")
    implementation("androidx.camera:camera-lifecycle:$cameraxVersion")
    implementation("androidx.camera:camera-view:$cameraxVersion")

    // Networking (OkHttp WebSockets + Coroutines)
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("com.google.code.gson:gson:2.10.1")

    // QR Code Scanning (ML Kit Barcode Scanning + Offline Standalone ZXing)
    implementation("com.google.mlkit:barcode-scanning:17.2.0")
    implementation("com.google.zxing:core:3.5.3")

    // Document Scanning (Google Play Services Document Scanner / Vision Edge)
    implementation("com.google.android.gms:play-services-mlkit-document-scanner:16.0.0")

    // Testing
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}
