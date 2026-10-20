package com.linkcam.app.ui

import android.view.Surface
import android.view.SurfaceHolder
import android.view.SurfaceView
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.linkcam.app.camera.CameraLens

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun StreamScreen(
    isConnected: Boolean,
    connectionMode: String,
    fps: Float,
    bitrateMbps: Float,
    latencyMs: Int,
    batteryPct: Int,
    currentLens: CameraLens,
    isTorchOn: Boolean,
    onToggleTorch: () -> Unit,
    onFlipCamera: () -> Unit,
    onTriggerDocumentScan: () -> Unit,
    onTriggerPhoto: () -> Unit,
    onEnterMeetingMode: () -> Unit,
    onDisconnect: () -> Unit,
    onSurfaceReady: (Surface) -> Unit = {},
    onSurfaceDestroyed: () -> Unit = {}
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF0A0E17))
    ) {
        // Native High-Performance Camera2 SurfaceView
        AndroidView(
            factory = { context ->
                SurfaceView(context).apply {
                    layoutParams = ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT
                    )
                    holder.addCallback(object : SurfaceHolder.Callback {
                        override fun surfaceCreated(holder: SurfaceHolder) {
                            onSurfaceReady(holder.surface)
                        }
                        override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {}
                        override fun surfaceDestroyed(holder: SurfaceHolder) {
                            onSurfaceDestroyed()
                        }
                    })
                }
            },
            modifier = Modifier.fillMaxSize()
        )

        // Top Status Bar Overlay (Window Insets Respecting)
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .statusBarsPadding()
                .padding(horizontal = 16.dp, vertical = 8.dp)
        ) {
            Surface(
                shape = RoundedCornerShape(24.dp),
                color = Color.Black.copy(alpha = 0.65f),
                tonalElevation = 6.dp,
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 14.dp, vertical = 8.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Left: Connection mode badge
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(8.dp)
                                .clip(CircleShape)
                                .background(if (isConnected) Color(0xFF10B981) else Color(0xFFEF4444))
                        )
                        Text(
                            text = connectionMode,
                            color = Color.White,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Medium
                        )
                    }

                    // Right: Telemetry metrics
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "${fps.toInt()} FPS",
                            color = Color(0xFFE2E8F0),
                            fontSize = 11.sp,
                            fontFamily = FontFamily.Monospace
                        )
                        Text(
                            text = "${"%.1f".format(bitrateMbps)}M",
                            color = Color(0xFF94A3B8),
                            fontSize = 11.sp,
                            fontFamily = FontFamily.Monospace
                        )
                        Text(
                            text = "${latencyMs}ms",
                            color = Color(0xFF94A3B8),
                            fontSize = 11.sp,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }
        }

        // Quick Side Tool Palette (Right Edge)
        Column(
            modifier = Modifier
                .align(Alignment.CenterEnd)
                .padding(end = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            // Torch Toggle
            IconButton(
                onClick = onToggleTorch,
                modifier = Modifier
                    .size(44.dp)
                    .background(
                        if (isTorchOn) Color(0xFFF59E0B).copy(alpha = 0.3f) else Color.Black.copy(alpha = 0.5f),
                        CircleShape
                    )
            ) {
                Icon(
                    imageVector = Icons.Rounded.FlashlightOn,
                    contentDescription = "Torch",
                    tint = if (isTorchOn) Color(0xFFFBBF24) else Color.White,
                    modifier = Modifier.size(20.dp)
                )
            }

            // Flip Camera Lens
            IconButton(
                onClick = onFlipCamera,
                modifier = Modifier
                    .size(44.dp)
                    .background(Color.Black.copy(alpha = 0.5f), CircleShape)
            ) {
                Icon(
                    imageVector = Icons.Rounded.FlipCameraAndroid,
                    contentDescription = "Flip Camera",
                    tint = Color.White,
                    modifier = Modifier.size(20.dp)
                )
            }

            // OLED Meeting Power-Save Mode
            IconButton(
                onClick = onEnterMeetingMode,
                modifier = Modifier
                    .size(44.dp)
                    .background(Color.Black.copy(alpha = 0.5f), CircleShape)
            ) {
                Icon(
                    imageVector = Icons.Rounded.DarkMode,
                    contentDescription = "Meeting Mode",
                    tint = Color(0xFFA855F7),
                    modifier = Modifier.size(20.dp)
                )
            }
        }

        // Bottom Ergonomic Action Dock
        Surface(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .navigationBarsPadding(),
            color = Color.Black.copy(alpha = 0.75f),
            shape = RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 20.dp, vertical = 16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Primary and Secondary Actions
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // Secondary Action: Snap Photo
                    OutlinedButton(
                        onClick = onTriggerPhoto,
                        modifier = Modifier
                            .weight(1f)
                            .height(52.dp),
                        shape = RoundedCornerShape(14.dp),
                        colors = ButtonDefaults.outlinedButtonColors(
                            contentColor = Color.White
                        )
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.PhotoCamera,
                            contentDescription = "Snap Photo",
                            modifier = Modifier.size(18.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(text = "Snap Photo", fontSize = 13.sp)
                    }

                    // Primary Action: Instant Document Scan
                    Button(
                        onClick = onTriggerDocumentScan,
                        modifier = Modifier
                            .weight(1.3f)
                            .height(52.dp),
                        shape = RoundedCornerShape(14.dp),
                        colors = ButtonDefaults.buttonColors(
                            containerColor = Color(0xFF2563EB)
                        )
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.DocumentScanner,
                            contentDescription = "Scan Document",
                            tint = Color.White,
                            modifier = Modifier.size(20.dp)
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = "Scan to PC",
                            color = Color.White,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.SemiBold
                        )
                    }
                }

                // Disconnect option
                TextButton(
                    onClick = onDisconnect,
                    modifier = Modifier.align(Alignment.CenterHorizontally)
                ) {
                    Text(
                        text = "Disconnect from Windows PC",
                        color = Color(0xFF94A3B8),
                        fontSize = 12.sp
                    )
                }
            }
        }
    }
}
