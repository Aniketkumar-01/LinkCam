package com.linkcam.app.camera

import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Bundle
import android.util.Log
import android.view.Surface
import java.util.concurrent.atomic.AtomicBoolean

/**
 * High-performance hardware-accelerated H.264 video encoder using Android MediaCodec Surface-to-Surface mode.
 * Camera ISP renders directly to inputSurface without CPU memory copying.
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
    private var drainThread: Thread? = null
    var inputSurface: Surface? = null
        private set

    init {
        initCodec()
    }

    private fun initCodec() {
        try {
            val format = MediaFormat.createVideoFormat(MIME_TYPE, width, height).apply {
                setInteger(MediaFormat.KEY_COLOR_FORMAT, MediaCodecInfo.CodecCapabilities.COLOR_FormatSurface)
                setInteger(MediaFormat.KEY_BIT_RATE, bitrateBps)
                setInteger(MediaFormat.KEY_FRAME_RATE, fps)
                setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1) // 1 second IDR keyframe interval
                setInteger(MediaFormat.KEY_BITRATE_MODE, MediaCodecInfo.EncoderCapabilities.BITRATE_MODE_CBR)
                try {
                    setInteger(MediaFormat.KEY_LATENCY, 0)
                    setInteger(MediaFormat.KEY_PRIORITY, 0)
                } catch (ignored: Exception) {}
            }

            val encoder = MediaCodec.createEncoderByType(MIME_TYPE)
            encoder.configure(format, null, null, MediaCodec.CONFIGURE_FLAG_ENCODE)
            inputSurface = encoder.createInputSurface()
            encoder.start()
            codec = encoder
            isRunning.set(true)

            // Dedicated low-latency buffer drainer
            drainThread = Thread({
                drainEncoderLoop()
            }, "LinkCamEncoderDrain").apply { start() }

            Log.i(TAG, "Hardware Surface-to-Surface H.264 MediaCodec initialized: ${width}x${height} @ ${fps}fps")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize hardware MediaCodec Surface encoder", e)
        }
    }

    private fun drainEncoderLoop() {
        val bufferInfo = MediaCodec.BufferInfo()
        while (isRunning.get()) {
            val encoder = codec ?: break
            try {
                val outputIndex = encoder.dequeueOutputBuffer(bufferInfo, TIMEOUT_USEC)
                if (outputIndex == MediaCodec.INFO_OUTPUT_FORMAT_CHANGED) {
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
                        onCodecConfig(configBytes)
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
                            onCodecConfig(outData)
                        } else {
                            onFrameEncoded(outData, isKeyframe, bufferInfo.presentationTimeUs / 1000L)
                        }
                    }
                    encoder.releaseOutputBuffer(outputIndex, false)
                }
            } catch (e: Exception) {
                if (!isRunning.get()) break
            }
        }
    }

    fun requestKeyframe() {
        val encoder = codec ?: return
        try {
            val params = Bundle().apply {
                putInt(MediaCodec.PARAMETER_KEY_REQUEST_SYNC_FRAME, 0)
            }
            encoder.setParameters(params)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to request sync keyframe", e)
        }
    }

    fun setBitrate(newBitrateBps: Int) {
        if (bitrateBps == newBitrateBps) return
        bitrateBps = newBitrateBps
        val encoder = codec ?: return
        try {
            val params = Bundle().apply {
                putInt(MediaCodec.PARAMETER_KEY_VIDEO_BITRATE, newBitrateBps)
            }
            encoder.setParameters(params)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to adjust bitrate dynamically", e)
        }
    }

    fun release() {
        isRunning.set(false)
        drainThread?.interrupt()
        try {
            codec?.stop()
            codec?.release()
        } catch (ignored: Exception) {}
        inputSurface?.release()
        inputSurface = null
        codec = null
    }
}
