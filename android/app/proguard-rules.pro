# LinkCam Android Proguard Rules
# Keep AndroidX & Lifecycle components
-keep class androidx.lifecycle.** { *; }
-keep class androidx.core.** { *; }

# Keep CameraX internals
-keep class androidx.camera.core.** { *; }
-keep class androidx.camera.camera2.** { *; }
-keep class androidx.camera.lifecycle.** { *; }
-keep class androidx.camera.view.** { *; }

# Keep Kotlin Coroutines
-keepnames class kotlinx.coroutines.** { *; }

# Keep Gson serialization models and protocol
-keepattributes Signature
-keepattributes *Annotation*
-keep class com.google.gson.** { *; }
-keep class com.linkcam.app.protocol.** { *; }
-keepclassmembers class com.linkcam.app.protocol.** { *; }

# Keep ML Kit Barcode and Document Scanner
-keep class com.google.mlkit.** { *; }
-keep class com.google.android.gms.vision.** { *; }
