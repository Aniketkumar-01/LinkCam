package com.linkcam.app.transport

import android.util.Log
import com.linkcam.app.protocol.PacketHeader
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * High-throughput UDP datagram video streamer.
 * Eliminates Head-Of-Line (HOL) blocking over Wi-Fi for ultra-low-latency video streaming.
 */
class UdpStreamer(
    private val remoteHost: String,
    private val remotePort: Int = 8444
) {
    companion object {
        private const val TAG = "UdpStreamer"
        // Maximum UDP packet size before IP fragmentation (typical MTU safe limit is ~1400 bytes,
        // or up to 64KB for local Wi-Fi jumbo frames)
        private const val MAX_UDP_CHUNK_SIZE = 60 * 1024
    }

    private var socket: DatagramSocket? = null
    private var remoteAddress: InetAddress? = null
    private val sequenceCounter = AtomicInteger(1)
    private val isRunning = AtomicBoolean(false)
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    fun start() {
        scope.launch {
            try {
                remoteAddress = InetAddress.getByName(remoteHost)
                socket = DatagramSocket()
                isRunning.set(true)
                Log.i(TAG, "UDP streamer started targeting $remoteHost:$remotePort")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start UDP socket", e)
            }
        }
    }

    fun sendVideoFrame(frameData: ByteArray, isKeyframe: Boolean, timestampMs: Long) {
        if (!isRunning.get()) return
        val sock = socket ?: return
        val targetAddr = remoteAddress ?: return

        scope.launch {
            try {
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_VIDEO_FRAME,
                    flags = if (isKeyframe) PacketHeader.FLAG_KEYFRAME else 0,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = timestampMs,
                    payloadLength = frameData.size
                )

                val headerBytes = header.toByteArray()
                val totalLength = headerBytes.size + frameData.size

                if (totalLength <= MAX_UDP_CHUNK_SIZE) {
                    val packetBuffer = ByteArray(totalLength)
                    System.arraycopy(headerBytes, 0, packetBuffer, 0, headerBytes.size)
                    System.arraycopy(frameData, 0, packetBuffer, headerBytes.size, frameData.size)

                    val packet = DatagramPacket(packetBuffer, totalLength, targetAddr, remotePort)
                    sock.send(packet)
                } else {
                    // For large IDR keyframes, send in sliced datagrams
                    var offset = 0
                    while (offset < frameData.size) {
                        val sliceSize = Math.min(MAX_UDP_CHUNK_SIZE - headerBytes.size, frameData.size - offset)
                        val sliceBuffer = ByteArray(headerBytes.size + sliceSize)
                        System.arraycopy(headerBytes, 0, sliceBuffer, 0, headerBytes.size)
                        System.arraycopy(frameData, offset, sliceBuffer, headerBytes.size, sliceSize)

                        val packet = DatagramPacket(sliceBuffer, sliceBuffer.size, targetAddr, remotePort)
                        sock.send(packet)
                        offset += sliceSize
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "Failed to send UDP datagram frame: ${e.message}")
            }
        }
    }

    fun stop() {
        isRunning.set(false)
        try {
            socket?.close()
        } catch (ignored: Exception) {}
        socket = null
    }
}
