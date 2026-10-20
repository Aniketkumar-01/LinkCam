package com.linkcam.app.camera

import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Bundle
import android.util.Log
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean

/**
 * High-performance hardware-accelerated H.264 video encoder using Android MediaCodec.
 * Delivers low-latency CBR video frames with on-the-fly Adaptive Bitrate (ABR) adjustments.
 */
class MediaCodecH264Encoder(
    private val width: Int = 1920,
    private val height: Int = 1080,
    private var bitrateBps: Int = 8_000_000,
    private val fps: Int = 60,
    private val onCodecConfig: (ByteArray) -> Unit,
    private val onFrameEncoded: (ByteArray, Boolean, Long) -> Unit
) {
    companion object {
        private const val TAG = "MediaCodecH264"
        private const val MIME_TYPE = "video/avc"
        private const val TIMEOUT_USEC = 10_000L
    }

    private var codec: MediaCodec? = null
    private val isRunning = AtomicBoolean(false)
    private var spsPpsData: ByteArray? = null

    init {
        initCodec()
    }

    private fun initCodec() {
        try {
            val format = MediaFormat.createVideoFormat(MIME_TYPE, width, height).apply {
                setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatYUV420Flexible)
                setInteger(MediaFormat.KEY_BIT_RATE, bitrateBps)
                setInteger(MediaFormat.KEY_FRAME_RATE, fps)
                setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1) // 1 second IDR keyframe interval
                setInteger(MediaFormat.KEY_BITRATE_MODE, MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)
                setInteger(MediaFormat.KEY_COMPLEXITY, MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)
                // Low-latency mode
                try {
                    setInteger(MediaFormat.KEY_LATENCY, 0)
                    setInteger(MediaFormat.KEY_PRIORITY, 0)
                } catch (ignored: Exception) {}
            }

            codec = MediaCodec.createEncoderByType(MIME_TYPE).apply {
                configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
                start()
            }
            isRunning.set(true)
            Log.i(TAG, "Hardware H.264 MediaCodec initialized: ${width}x${height} @ ${fps}fps, ${bitrateBps / 1000} kbps")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize hardware MediaCodec encoder", e)
        }
    }

    /**
     * Feed raw YUV420 frame data into the hardware encoder
     */
    fun encodeFrame(yuvData: ByteArray, timestampNs: Long) {
        if (!isRunning.get()) return
        val encoder = codec ?: return

        try {
            val inputIndex = encoder.dequeueInputBuffer(TIMEOUT_USEC)
            if (inputIndex >= 0) {
                val inputBuffer = encoder.getInputBuffer(inputIndex)
                inputBuffer?.clear()
                val bytesToCopy = Math.min(yuvData.size, inputBuffer?.remaining() ?: 0)
                inputBuffer?.put(yuvData, 0, bytesToCopy)
                val ptsUs = timestampNs / 1000L
                encoder.queueInputBuffer(inputIndex, 0, bytesToCopy, ptsUs, 0)
            }

            drainEncoder()
        } catch (e: Exception) {
            Log.e(TAG, "Error encoding frame", e)
        }
    }

    private fun drainEncoder() {
        val encoder = codec ?: return
        val bufferInfo = MediaCodec.BufferInfo()

        while (isRunning.get()) {
            val outputIndex = encoder.dequeueOutputBuffer(bufferInfo, 0)
            if (outputIndex == MediaCodec.INFO_TRY_AGAIN_LATER) {
                break
            } else if (outputIndex == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED) {
                val newFormat = encoder.outputFormat
                val sps = newFormat.getByteBuffer("csd-0")
                val pps = newFormat.getByteBuffer("csd-1")
                if (sps != null && pps != null) {
                    val spsBytes = ByteArray(sps.remaining()).also { sps.get(it) }
                    val ppsBytes = ByteArray(pps.remaining()).also { pps.get(it) }
                    val configBytes = ByteArray(spsBytes.size + ppsBytes.size).apply {
                        System.arraycopy(spsBytes, 0, this, 0, spsBytes.size)
                        System.arraycopy(ppsBytes, 0, this, spsBytes.size, ppsBytes.size)
                    }
                    spsPpsData = configBytes
                    onCodecConfig(configBytes)
                    Log.i(TAG, "H.264 SPS/PPS parameters extracted: ${configBytes.size} bytes")
                }
            } else if (outputIndex >= 0) {
                val outputBuffer = encoder.getOutputBuffer(outputIndex)
                if (outputBuffer != null && bufferInfo.size > 0) {
                    outputBuffer.position(bufferInfo.offset)
                    outputBuffer.limit(bufferInfo.offset + bufferInfo.size)

                    val outData = ByteArray(bufferInfo.size)
                    outputBuffer.get(outData)

                    val isKeyframe = (bufferInfo.flags and MediaCodec.BUFFER_FLAG_KEY_FRAME) != 0
                    val isConfig = (bufferInfo.flags and MediaCodec.BUFFER_FLAG_CODEC_CONFIG) != 0

                    if (isConfig) {
                        spsPpsData = outData
                        onCodecConfig(outData)
                    } else {
                        onFrameEncoded(outData, isKeyframe, bufferInfo.presentationTimeUs / 1000L)
                    }
                }
                encoder.releaseOutputBuffer(outputIndex, false)
            }
        }
    }

    /**
     * Request an Instantaneous Decoder Refresh (IDR) keyframe immediately
     */
    fun requestKeyframe() {
        val encoder = codec ?: return
        try {
            val params = Bundle().apply {
                putInt(MediaCodec.PARAMETER_KEY_REQUEST_SYNC_FRAME, 0)
            }
            encoder.setParameters(params)
            Log.d(TAG, "IDR sync keyframe requested")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to request sync keyframe", e)
        }
    }

    /**
     * Dynamically adjust encoder bitrate for Adaptive Bitrate (ABR) control
     */
    fun setBitrate(newBitrateBps: Int) {
        if (bitrateBps == newBitrateBps) return
        bitrateBps = newBitrateBps
        val encoder = codec ?: return
        try {
            val params = Bundle().apply {
                putInt(MediaCodec.PARAMETER_KEY_VIDEO_BITRATE, newBitrateBps)
            }
            encoder.setParameters(params)
            Log.i(TAG, "Dynamically updated H.264 bitrate to ${newBitrateBps / 1000} kbps")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to adjust bitrate dynamically", e)
        }
    }

    fun release() {
        isRunning.set(false)
        try {
            codec?.stop()
            codec?.release()
        } catch (e: Exception) {
            Log.e(TAG, "Error releasing MediaCodec", e)
        }
        codec = null
    }
}
