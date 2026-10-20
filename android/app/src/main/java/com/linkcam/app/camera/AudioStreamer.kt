package com.linkcam.app.camera

import android.annotation.SuppressLint
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.MediaRecorder
import android.util.Log
import com.linkcam.app.protocol.Packet
import com.linkcam.app.protocol.PacketHeader
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * Low-latency microphone audio capture streamer.
 * Records 48 kHz 16-bit PCM stereo audio with monotonic Presentation Timestamps (PTS) for A/V synchronization.
 */
class AudioStreamer(
    private val onAudioPacketEncoded: (Packet) -> Unit
) {
    companion object {
        private const val TAG = "AudioStreamer"
        const val SAMPLE_RATE = 48000
        const val CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_STEREO
        const val AUDIO_FORMAT = AudioFormat.ENCODING_PCM_16BIT
    }

    private var audioRecord: AudioRecord? = null
    private var recordingThread: Thread? = null
    private val isRecording = AtomicBoolean(false)
    private val sequenceCounter = AtomicInteger(1)

    @SuppressLint("MissingPermission")
    fun start() {
        val minBufferSize = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT)
        val bufferSize = Math.max(minBufferSize, 4096)

        try {
            audioRecord = AudioRecord(
                MediaRecorder.AudioSource.VOICE_COMMUNICATION,
                SAMPLE_RATE,
                CHANNEL_CONFIG,
                AUDIO_FORMAT,
                bufferSize
            )

            if (audioRecord?.state != AudioRecord.STATE_INITIALIZED) {
                Log.e(TAG, "AudioRecord failed to initialize")
                return
            }

            audioRecord?.startRecording()
            isRecording.set(true)

            recordingThread = Thread({
                val buffer = ByteArray(2048)
                while (isRecording.get()) {
                    val read = audioRecord?.read(buffer, 0, buffer.size) ?: -1
                    if (read > 0) {
                        val timestampMs = System.nanoTime() / 1_000_000L
                        val payload = ByteArray(read)
                        System.arraycopy(buffer, 0, payload, 0, read)

                        val header = PacketHeader(
                            packetType = PacketHeader.TYPE_AUDIO_FRAME,
                            sequence = sequenceCounter.getAndIncrement(),
                            timestamp = timestampMs,
                            payloadLength = read
                        )
                        val packet = Packet(header, payload)
                        onAudioPacketEncoded(packet)
                    }
                }
            }, "LinkCamAudioCaptureThread").apply { start() }

            Log.i(TAG, "AudioStreamer recording started (48kHz stereo PCM)")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start AudioRecord", e)
        }
    }

    fun stop() {
        isRecording.set(false)
        try {
            recordingThread?.join(500)
            audioRecord?.stop()
            audioRecord?.release()
        } catch (e: Exception) {
            Log.e(TAG, "Error stopping AudioRecord", e)
        }
        audioRecord = null
        recordingThread = null
    }
}
