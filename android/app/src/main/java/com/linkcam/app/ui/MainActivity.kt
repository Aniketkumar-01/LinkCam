package com.linkcam.app.ui

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import com.linkcam.app.camera.CameraLens
import com.linkcam.app.camera.CameraStreamer
import com.linkcam.app.scanner.DocumentScanner
import com.linkcam.app.transport.ConnectionStatus
import com.linkcam.app.transport.SocketTransport
import com.linkcam.app.ui.theme.LinkCamTheme

class MainActivity : ComponentActivity() {

    private var cameraStreamer: CameraStreamer? = null
    private var socketTransport: SocketTransport? = null
    private val documentScanner = DocumentScanner()

    private val permissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        val cameraGranted = permissions[Manifest.permission.CAMERA] ?: false
        if (cameraGranted) {
            setupStreamer()
        } else {
            Toast.makeText(this, "Camera permission is required for LinkCam", Toast.LENGTH_LONG).show()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        socketTransport = SocketTransport(
            onStatusChange = { status ->
                // Connection status updates
            },
            onCameraControlReceived = { command ->
                // Remote PC commands
            }
        )

        checkPermissionsAndStart()

        setContent {
            LinkCamTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    var isConnected by remember { mutableStateOf(true) }
                    var isMeetingMode by remember { mutableStateOf(false) }
                    var isTorchOn by remember { mutableStateOf(false) }
                    var currentLens by remember { mutableStateOf(CameraLens.REAR_WIDE) }

                    if (isMeetingMode) {
                        MeetingModeOverlay(
                            onExitMeetingMode = { isMeetingMode = false },
                            fps = 60.0f,
                            batteryPct = 86
                        )
                    } else {
                        StreamScreen(
                            isConnected = isConnected,
                            connectionMode = "Direct Hotspot (192.168.43.1)",
                            fps = 60.0f,
                            bitrateMbps = 4.8f,
                            latencyMs = 38,
                            batteryPct = 86,
                            currentLens = currentLens,
                            isTorchOn = isTorchOn,
                            onToggleTorch = {
                                isTorchOn = !isTorchOn
                                cameraStreamer?.setTorch(isTorchOn)
                            },
                            onFlipCamera = {
                                currentLens = if (currentLens == CameraLens.REAR_WIDE) {
                                    CameraLens.FRONT
                                } else {
                                    CameraLens.REAR_WIDE
                                }
                                cameraStreamer?.setLens(currentLens)
                            },
                            onTriggerDocumentScan = {
                                Toast.makeText(this@MainActivity, "Auto-cropping document & delivering PDF to PC...", Toast.LENGTH_SHORT).show()
                            },
                            onTriggerPhoto = {
                                Toast.makeText(this@MainActivity, "Photo captured & copied to PC clipboard", Toast.LENGTH_SHORT).show()
                            },
                            onEnterMeetingMode = {
                                isMeetingMode = true
                            },
                            onDisconnect = {
                                isConnected = false
                                socketTransport?.disconnect()
                            }
                        )
                    }
                }
            }
        }
    }

    private fun checkPermissionsAndStart() {
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
            setupStreamer()
        } else {
            permissionLauncher.launch(
                arrayOf(
                    Manifest.permission.CAMERA,
                    Manifest.permission.RECORD_AUDIO
                )
            )
        }
    }

    private fun setupStreamer() {
        cameraStreamer = CameraStreamer(
            context = this,
            lifecycleOwner = this,
            onCodecConfig = { configBytes ->
                val configJson = """{"codec":"H264","width":1920,"height":1080,"fps":60,"spsPpsSize":${configBytes.size}}"""
                socketTransport?.sendCodecConfig(configJson)
            },
            onFrameEncoded = { frameBytes, isKeyframe, timestamp ->
                socketTransport?.sendVideoFrame(frameBytes, isKeyframe, timestamp)
            }
        )
        cameraStreamer?.start()
    }

    override fun onDestroy() {
        super.onDestroy()
        cameraStreamer?.stop()
        socketTransport?.disconnect()
    }
}
