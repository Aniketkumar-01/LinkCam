package com.linkcam.app.ui

import android.view.ViewGroup
import androidx.annotation.OptIn
import androidx.camera.core.CameraSelector
import androidx.camera.core.ExperimentalGetImage
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import android.util.Size
import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.common.InputImage
import com.google.zxing.BarcodeFormat
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.MultiFormatReader
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.common.HybridBinarizer
import java.util.concurrent.Executors

@Composable
fun PairingScreen(
    deviceName: String,
    batteryPct: Int,
    isConnecting: Boolean,
    errorMessage: String?,
    onConnect: (host: String, port: Int, token: String, sessionId: String) -> Unit
) {
    var activeTab by remember { mutableStateOf(0) } // 0: QR Scan, 1: Manual / USB
    var manualIp by remember { mutableStateOf("") }
    var manualPort by remember { mutableStateOf("8443") }
    var manualPin by remember { mutableStateOf("") }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF0A0E17))
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding()
                .padding(horizontal = 20.dp, vertical = 12.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Top Header: App Branding & Real Telemetry
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = 8.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .clip(RoundedCornerShape(10.dp))
                            .background(Color(0xFF2563EB)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.Videocam,
                            contentDescription = "LinkCam",
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                    Column {
                        Text(
                            text = "LinkCam",
                            fontSize = 18.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color.White
                        )
                        Text(
                            text = deviceName,
                            fontSize = 12.sp,
                            color = Color(0xFF94A3B8)
                        )
                    }
                }

                // Real Battery Pill
                Surface(
                    shape = RoundedCornerShape(12.dp),
                    color = Color.White.copy(alpha = 0.06f)
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.BatteryChargingFull,
                            contentDescription = "Battery",
                            tint = if (batteryPct > 20) Color(0xFF10B981) else Color(0xFFEF4444),
                            modifier = Modifier.size(16.dp)
                        )
                        Text(
                            text = "$batteryPct%",
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Medium,
                            color = Color.White
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Tab Selector: Scan QR vs Manual/USB
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = Color.White.copy(alpha = 0.05f),
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(modifier = Modifier.padding(4.dp)) {
                    Button(
                        onClick = { activeTab = 0 },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = if (activeTab == 0) Color(0xFF2563EB) else Color.Transparent,
                            contentColor = if (activeTab == 0) Color.White else Color(0xFF94A3B8)
                        ),
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.weight(1f),
                        contentPadding = PaddingValues(vertical = 10.dp)
                    ) {
                        Icon(Icons.Rounded.QrCodeScanner, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Scan QR Code", fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                    }

                    Button(
                        onClick = { activeTab = 1 },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = if (activeTab == 1) Color(0xFF2563EB) else Color.Transparent,
                            contentColor = if (activeTab == 1) Color.White else Color(0xFF94A3B8)
                        ),
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.weight(1f),
                        contentPadding = PaddingValues(vertical = 10.dp)
                    ) {
                        Icon(Icons.Rounded.SettingsEthernet, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("Manual / USB", fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }

            Spacer(modifier = Modifier.height(20.dp))

            // Main Content Area
            if (activeTab == 0) {
                // QR Scanner Viewfinder
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .clip(RoundedCornerShape(24.dp))
                        .border(1.dp, Color.White.copy(alpha = 0.1f), RoundedCornerShape(24.dp)),
                    contentAlignment = Alignment.Center
                ) {
                    QrCameraScanner(
                        onQrScanned = { qrContent ->
                            parseAndConnect(qrContent, onConnect)
                        }
                    )

                    // Reticle overlay
                    Box(
                        modifier = Modifier
                            .size(240.dp)
                            .border(2.dp, Color(0xFF38BDF8), RoundedCornerShape(20.dp))
                    )

                    // Helper label at bottom of scanner
                    Surface(
                        shape = RoundedCornerShape(16.dp),
                        color = Color.Black.copy(alpha = 0.75f),
                        modifier = Modifier
                            .align(Alignment.BottomCenter)
                            .padding(bottom = 20.dp)
                    ) {
                        Text(
                            text = "Point camera at LinkCam QR code on PC",
                            color = Color.White,
                            fontSize = 12.sp,
                            modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp)
                        )
                    }
                }
            } else {
                // Manual IP & USB Connect Area
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .weight(1f)
                        .padding(vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    var usbPin by remember { mutableStateOf("") }

                    // USB Cable Connect Card
                    Surface(
                        shape = RoundedCornerShape(20.dp),
                        color = Color(0xFF1E293B).copy(alpha = 0.7f),
                        border = androidx.compose.foundation.BorderStroke(1.dp, Color.White.copy(alpha = 0.08f)),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Column(modifier = Modifier.padding(18.dp)) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(10.dp)
                            ) {
                                Box(
                                    modifier = Modifier
                                        .size(36.dp)
                                        .clip(CircleShape)
                                        .background(Color(0xFF10B981).copy(alpha = 0.15f)),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Icon(
                                        imageVector = Icons.Rounded.Usb,
                                        contentDescription = "USB",
                                        tint = Color(0xFF10B981),
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                                Column {
                                    Text(
                                        text = "USB Cable Tethering",
                                        fontSize = 14.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        color = Color.White
                                    )
                                    Text(
                                        text = "Ultra-low latency 15–30ms (tcp:8443 via ADB)",
                                        fontSize = 12.sp,
                                        color = Color(0xFF94A3B8)
                                    )
                                }
                            }

                            Spacer(modifier = Modifier.height(10.dp))

                            Text(
                                text = "Zero static bypass: Desktop companion pushes a one-time token over ADB, or enter Desktop PIN below.",
                                fontSize = 11.sp,
                                color = Color(0xFF94A3B8)
                            )

                            Spacer(modifier = Modifier.height(10.dp))

                            OutlinedTextField(
                                value = usbPin,
                                onValueChange = { usbPin = it },
                                label = { Text("Desktop PIN (or wait for ADB auto-push)") },
                                placeholder = { Text("e.g. 582 391") },
                                singleLine = true,
                                colors = OutlinedTextFieldDefaults.colors(
                                    focusedBorderColor = Color(0xFF10B981),
                                    unfocusedBorderColor = Color.White.copy(alpha = 0.2f),
                                    focusedTextColor = Color.White,
                                    unfocusedTextColor = Color.White
                                ),
                                modifier = Modifier.fillMaxWidth()
                            )

                            Spacer(modifier = Modifier.height(12.dp))

                            Button(
                                onClick = {
                                    val token = if (usbPin.isNotBlank()) usbPin.trim() else "usb_auth_request"
                                    onConnect("127.0.0.1", 8443, token, "usb_session")
                                },
                                colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF10B981)),
                                shape = RoundedCornerShape(12.dp),
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Icon(Icons.Rounded.Cable, contentDescription = null, modifier = Modifier.size(18.dp))
                                Spacer(modifier = Modifier.width(8.dp))
                                Text("Connect via USB (127.0.0.1)", fontWeight = FontWeight.Bold)
                            }
                        }
                    }

                    // Direct Wi-Fi IP Connect Card
                    Surface(
                        shape = RoundedCornerShape(20.dp),
                        color = Color(0xFF1E293B).copy(alpha = 0.7f),
                        border = androidx.compose.foundation.BorderStroke(1.dp, Color.White.copy(alpha = 0.08f)),
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Column(
                            modifier = Modifier.padding(18.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            Text(
                                text = "Direct Wi-Fi / Hotspot Connect",
                                fontSize = 14.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = Color.White
                            )

                            OutlinedTextField(
                                value = manualIp,
                                onValueChange = { manualIp = it },
                                label = { Text("PC IP Address (e.g. 192.168.1.100)") },
                                placeholder = { Text("192.168.1.xxx") },
                                singleLine = true,
                                modifier = Modifier.fillMaxWidth(),
                                colors = OutlinedTextFieldDefaults.colors(
                                    focusedBorderColor = Color(0xFF2563EB),
                                    unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
                                    focusedTextColor = Color.White,
                                    unfocusedTextColor = Color.White
                                )
                            )

                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.spacedBy(8.dp)
                            ) {
                                OutlinedTextField(
                                    value = manualPort,
                                    onValueChange = { manualPort = it },
                                    label = { Text("Port") },
                                    singleLine = true,
                                    modifier = Modifier.weight(1f),
                                    colors = OutlinedTextFieldDefaults.colors(
                                        focusedBorderColor = Color(0xFF2563EB),
                                        unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
                                        focusedTextColor = Color.White,
                                        unfocusedTextColor = Color.White
                                    )
                                )

                                OutlinedTextField(
                                    value = manualPin,
                                    onValueChange = { manualPin = it },
                                    label = { Text("6-Digit PIN") },
                                    singleLine = true,
                                    modifier = Modifier.weight(1.5f),
                                    colors = OutlinedTextFieldDefaults.colors(
                                        focusedBorderColor = Color(0xFF2563EB),
                                        unfocusedBorderColor = Color.White.copy(alpha = 0.15f),
                                        focusedTextColor = Color.White,
                                        unfocusedTextColor = Color.White
                                    )
                                )
                            }

                            Button(
                                onClick = {
                                    val port = manualPort.toIntOrNull() ?: 8443
                                    if (manualIp.isNotBlank()) {
                                        onConnect(manualIp.trim(), port, manualPin.trim(), "manual_session")
                                    }
                                },
                                enabled = manualIp.isNotBlank(),
                                colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF2563EB)),
                                shape = RoundedCornerShape(12.dp),
                                modifier = Modifier.fillMaxWidth()
                            ) {
                                Text("Connect to PC", fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
            }

            // Status & Error Feedback
            if (isConnecting) {
                Spacer(modifier = Modifier.height(12.dp))
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = Color(0xFF38BDF8),
                        strokeWidth = 2.dp
                    )
                    Text(
                        text = "Connecting to Windows PC...",
                        fontSize = 13.sp,
                        color = Color(0xFF38BDF8)
                    )
                }
            }

            if (!errorMessage.isNullOrBlank()) {
                Spacer(modifier = Modifier.height(12.dp))
                Surface(
                    shape = RoundedCornerShape(12.dp),
                    color = Color(0xFFEF4444).copy(alpha = 0.15f),
                    border = androidx.compose.foundation.BorderStroke(1.dp, Color(0xFFEF4444).copy(alpha = 0.3f)),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = errorMessage,
                        color = Color(0xFFFCA5A5),
                        fontSize = 12.sp,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.padding(10.dp)
                    )
                }
            }
        }
    }
}

private fun parseAndConnect(
    qrContent: String,
    onConnect: (host: String, port: Int, token: String, sessionId: String) -> Unit
) {
    try {
        val gson = Gson()
        val json = gson.fromJson(qrContent, JsonObject::class.java)

        val port = if (json.has("port")) json.get("port").asInt
            else if (json.has("p")) json.get("p").asInt
            else 8443

        val token = if (json.has("token")) json.get("token").asString
            else if (json.has("t")) json.get("t").asString
            else ""

        val pin = if (json.has("pin")) json.get("pin").asString else ""

        val session = if (json.has("session_id")) json.get("session_id").asString
            else if (json.has("s")) json.get("s").asString
            else "qr_session"

        val ips = if (json.has("ips")) {
            json.getAsJsonArray("ips").map { it.asString }
        } else {
            emptyList()
        }

        // Connect to first non-loopback IP, or fallback to first IP
        val targetIp = ips.firstOrNull { it != "127.0.0.1" } ?: ips.firstOrNull() ?: "192.168.1.1"
        onConnect(targetIp, port, token.ifEmpty { pin }, session)
    } catch (e: Exception) {
        e.printStackTrace()
    }
}

@Composable
fun QrCameraScanner(
    onQrScanned: (String) -> Unit
) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val cameraExecutor = remember { Executors.newSingleThreadExecutor() }
    var cameraProviderRef by remember { mutableStateOf<ProcessCameraProvider?>(null) }
    var hasDetectedQr by remember { mutableStateOf(false) }

    DisposableEffect(Unit) {
        onDispose {
            try {
                cameraProviderRef?.unbindAll()
                cameraExecutor.shutdown()
            } catch (_: Exception) {}
        }
    }

    val handleScanned: (String) -> Unit = { raw ->
        if (!hasDetectedQr) {
            hasDetectedQr = true
            // Immediately unbind all CameraX use cases on main thread so hardware camera 0 is released
            ContextCompat.getMainExecutor(context).execute {
                try {
                    cameraProviderRef?.unbindAll()
                } catch (_: Exception) {}
                onQrScanned(raw)
            }
        }
    }

    AndroidView(
        factory = { ctx ->
            val previewView = PreviewView(ctx).apply {
                scaleType = PreviewView.ScaleType.FILL_CENTER
                layoutParams = ViewGroup.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT,
                    ViewGroup.LayoutParams.MATCH_PARENT
                )
            }

            val cameraProviderFuture = ProcessCameraProvider.getInstance(ctx)
            cameraProviderFuture.addListener({
                val cameraProvider = cameraProviderFuture.get()
                cameraProviderRef = cameraProvider

                if (hasDetectedQr) {
                    return@addListener
                }

                val preview = Preview.Builder().build().also {
                    it.setSurfaceProvider(previewView.surfaceProvider)
                }

                // Use 1280x720 for crisp QR code resolution
                val imageAnalysis = ImageAnalysis.Builder()
                    .setTargetResolution(Size(1280, 720))
                    .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                    .build()
                    .also { analysis ->
                        analysis.setAnalyzer(cameraExecutor, QrBarcodeAnalyzer(handleScanned))
                    }

                try {
                    cameraProvider.unbindAll()
                    val camera = cameraProvider.bindToLifecycle(
                        lifecycleOwner,
                        CameraSelector.DEFAULT_BACK_CAMERA,
                        preview,
                        imageAnalysis
                    )

                    // Continuous auto-focus on center
                    try {
                        val factory = androidx.camera.core.SurfaceOrientedMeteringPointFactory(1f, 1f)
                        val centerPoint = factory.createPoint(0.5f, 0.5f)
                        val action = androidx.camera.core.FocusMeteringAction.Builder(
                            centerPoint,
                            androidx.camera.core.FocusMeteringAction.FLAG_AF
                        ).build()
                        camera.cameraControl.startFocusAndMetering(action)
                    } catch (_: Exception) {}

                    // Tap to focus
                    previewView.setOnTouchListener { view, motionEvent ->
                        if (motionEvent.action == android.view.MotionEvent.ACTION_UP) {
                            val factory = previewView.meteringPointFactory
                            val point = factory.createPoint(motionEvent.x, motionEvent.y)
                            val action = androidx.camera.core.FocusMeteringAction.Builder(point).build()
                            camera.cameraControl.startFocusAndMetering(action)
                            view.performClick()
                        }
                        true
                    }
                } catch (e: Exception) {
                    e.printStackTrace()
                }
            }, ContextCompat.getMainExecutor(ctx))

            previewView
        },
        modifier = Modifier.fillMaxSize()
    )
}

@OptIn(ExperimentalGetImage::class)
class QrBarcodeAnalyzer(
    private val onQrScanned: (String) -> Unit
) : ImageAnalysis.Analyzer {
    private val isScanningActive = java.util.concurrent.atomic.AtomicBoolean(true)

    private val mlKitScanner = BarcodeScanning.getClient(
        BarcodeScannerOptions.Builder()
            .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
            .build()
    )
    private val zxingReader = MultiFormatReader().apply {
        setHints(mapOf(
            DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.QR_CODE),
            DecodeHintType.TRY_HARDER to true
        ))
    }

    private var lastScannedTimestamp = 0L

    override fun analyze(imageProxy: ImageProxy) {
        if (!isScanningActive.get()) {
            imageProxy.close()
            return
        }

        val now = System.currentTimeMillis()
        if (now - lastScannedTimestamp < 2500) {
            imageProxy.close()
            return
        }

        val mediaImage = imageProxy.image
        if (mediaImage != null) {
            val rotation = imageProxy.imageInfo.rotationDegrees
            val inputImage = InputImage.fromMediaImage(mediaImage, rotation)

            mlKitScanner.process(inputImage)
                .addOnSuccessListener { barcodes ->
                    var found = false
                    for (barcode in barcodes) {
                        val raw = barcode.rawValue
                        if (!raw.isNullOrBlank()) {
                            found = true
                            lastScannedTimestamp = System.currentTimeMillis()
                            if (isScanningActive.compareAndSet(true, false)) {
                                onQrScanned(raw)
                            }
                            break
                        }
                    }
                    if (!found && isScanningActive.get()) {
                        tryZxingDecode(imageProxy)
                    }
                }
                .addOnFailureListener {
                    if (isScanningActive.get()) {
                        tryZxingDecode(imageProxy)
                    }
                }
                .addOnCompleteListener {
                    imageProxy.close()
                }
        } else {
            if (isScanningActive.get()) {
                tryZxingDecode(imageProxy)
            }
            imageProxy.close()
        }
    }

    private fun tryZxingDecode(imageProxy: ImageProxy) {
        try {
            val plane = imageProxy.planes[0]
            val buffer = plane.buffer
            val bytes = ByteArray(buffer.remaining())
            buffer.get(bytes)

            val width = imageProxy.width
            val height = imageProxy.height
            val rowStride = plane.rowStride

            val source = PlanarYUVLuminanceSource(
                bytes, rowStride, height, 0, 0, width, height, false
            )
            val binaryBitmap = BinaryBitmap(HybridBinarizer(source))
            val result = zxingReader.decodeWithState(binaryBitmap)

            if (!result.text.isNullOrBlank()) {
                lastScannedTimestamp = System.currentTimeMillis()
                if (isScanningActive.compareAndSet(true, false)) {
                    onQrScanned(result.text)
                }
            }
        } catch (_: Exception) {
            // Frame does not contain QR
        }
    }
}
