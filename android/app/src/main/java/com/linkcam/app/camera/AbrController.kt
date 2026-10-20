package com.linkcam.app.camera

import android.util.Log

enum class StreamQualityState {
    NOMINAL,
    DEGRADED,
    EMERGENCY
}

/**
 * Adaptive Bitrate (ABR) state machine.
 * Monitors network round-trip time, packet loss %, and thermal status to dynamically scale video bandwidth.
 */
class AbrController(
    private val onBitrateAdjusted: (Int, Int) -> Unit // (bitrateBps, targetFps)
) {
    companion object {
        private const val TAG = "AbrController"
        const val BITRATE_NOMINAL = 8_000_000   // 8 Mbps
        const val BITRATE_DEGRADED = 4_500_000  // 4.5 Mbps
        const val BITRATE_EMERGENCY = 2_200_000 // 2.2 Mbps
    }

    private var currentState: StreamQualityState = StreamQualityState.NOMINAL
    private var consecutiveGoodTicks = 0

    fun updateMetrics(rttMs: Long, packetLossPct: Float, isThermalThrottled: Boolean) {
        val previousState = currentState

        if (packetLossPct > 7.0f || isThermalThrottled) {
            currentState = StreamQualityState.EMERGENCY
            consecutiveGoodTicks = 0
        } else if (packetLossPct > 2.5f || rttMs > 50) {
            if (currentState != StreamQualityState.EMERGENCY) {
                currentState = StreamQualityState.DEGRADED
            }
            consecutiveGoodTicks = 0
        } else if (packetLossPct < 0.5f && rttMs < 25) {
            consecutiveGoodTicks++
            if (consecutiveGoodTicks >= 10) { // 10 seconds of stable network
                currentState = when (currentState) {
                    StreamQualityState.EMERGENCY -> StreamQualityState.DEGRADED
                    StreamQualityState.DEGRADED -> StreamQualityState.NOMINAL
                    StreamQualityState.NOMINAL -> StreamQualityState.NOMINAL
                }
                consecutiveGoodTicks = 0
            }
        }

        if (currentState != previousState) {
            Log.i(TAG, "ABR Transition: $previousState -> $currentState (RTT=${rttMs}ms, Loss=${packetLossPct}%)")
            when (currentState) {
                StreamQualityState.NOMINAL -> onBitrateAdjusted(BITRATE_NOMINAL, 60)
                StreamQualityState.DEGRADED -> onBitrateAdjusted(BITRATE_DEGRADED, 60)
                StreamQualityState.EMERGENCY -> onBitrateAdjusted(BITRATE_EMERGENCY, 30)
            }
        }
    }

    fun getCurrentState(): StreamQualityState = currentState
}
