package com.linkcam.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Videocam
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun MeetingModeOverlay(
    onExitMeetingMode: () -> Unit,
    fps: Float,
    batteryPct: Int
) {
    // Pure black screen for OLED power saving & zero thermal heating during long calls
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black)
            .clickable(
                indication = null,
                interactionSource = remember { MutableInteractionSource() }
            ) {
                onExitMeetingMode()
            },
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Icon(
                imageVector = Icons.Rounded.Videocam,
                contentDescription = "Meeting Mode Active",
                tint = Color(0xFF10B981).copy(alpha = 0.5f),
                modifier = Modifier.size(36.dp)
            )

            Text(
                text = "Meeting Mode Active",
                color = Color.White.copy(alpha = 0.6f),
                fontSize = 14.sp,
                fontWeight = FontWeight.Medium
            )

            Text(
                text = "Streaming in background • Tap anywhere to wake",
                color = Color.Gray.copy(alpha = 0.5f),
                fontSize = 12.sp
            )

            Row(
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                modifier = Modifier.padding(top = 8.dp)
            ) {
                Text(
                    text = "${fps.toInt()} FPS",
                    color = Color.White.copy(alpha = 0.4f),
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace
                )
                Text(
                    text = "🔋 $batteryPct%",
                    color = Color.White.copy(alpha = 0.4f),
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace
                )
            }
        }
    }
}
