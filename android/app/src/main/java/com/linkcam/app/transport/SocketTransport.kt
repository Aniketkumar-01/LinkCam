package com.linkcam.app.transport

import com.google.gson.Gson
import com.google.gson.JsonObject
import com.linkcam.app.protocol.Packet
import com.linkcam.app.protocol.PacketHeader
import kotlinx.coroutines.*
import java.io.InputStream
import java.io.OutputStream
import java.net.InetSocketAddress
import java.net.Socket
import java.nio.ByteBuffer
import java.util.concurrent.atomic.AtomicInteger

enum class ConnectionStatus {
    DISCONNECTED,
    CONNECTING,
    AUTHENTICATING,
    CONNECTED
}

class SocketTransport(
    private val onStatusChange: (ConnectionStatus) -> Unit,
    private val onCameraControlReceived: (JsonObject) -> Unit
) {
    private var socket: Socket? = null
    private var outputStream: OutputStream? = null
    private var inputStream: InputStream? = null
    private val sequenceCounter = AtomicInteger(1)
    private var isRunning = false
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val gson = Gson()

    fun connect(host: String, port: Int, token: String, sessionId: String) {
        scope.launch {
            try {
                onStatusChange(ConnectionStatus.CONNECTING)
                val sock = Socket()
                sock.connect(InetSocketAddress(host, port), 5000)
                socket = sock
                outputStream = sock.getOutputStream()
                inputStream = sock.getInputStream()

                // Authenticate
                onStatusChange(ConnectionStatus.AUTHENTICATING)
                val authPayload = "$token:$sessionId".toByteArray(Charsets.UTF_8)
                val authHeader = PacketHeader(
                    packetType = PacketHeader.TYPE_PAIRING_AUTH,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = System.currentTimeMillis(),
                    payloadLength = authPayload.size
                )
                val authPacket = Packet(authHeader, authPayload)
                outputStream?.write(authPacket.toByteArray())
                outputStream?.flush()

                // Read Auth Response (24 bytes header + payload)
                val respHeaderBytes = ByteArray(PacketHeader.HEADER_SIZE)
                inputStream?.read(respHeaderBytes)
                val respHeader = PacketHeader.readFrom(ByteBuffer.wrap(respHeaderBytes))

                if (respHeader.packetType == PacketHeader.TYPE_PAIRING_AUTH_RESPONSE) {
                    val respPayload = ByteArray(respHeader.payloadLength)
                    inputStream?.read(respPayload)
                    if (String(respPayload) == "OK") {
                        isRunning = true
                        onStatusChange(ConnectionStatus.CONNECTED)
                        startReceiveLoop()
                    } else {
                        disconnect()
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
                disconnect()
            }
        }
    }

    private fun startReceiveLoop() {
        scope.launch {
            val headerBuffer = ByteArray(PacketHeader.HEADER_SIZE)
            while (isRunning) {
                try {
                    val bytesRead = inputStream?.read(headerBuffer) ?: -1
                    if (bytesRead < PacketHeader.HEADER_SIZE) break

                    val header = PacketHeader.readFrom(ByteBuffer.wrap(headerBuffer))
                    val payload = ByteArray(header.payloadLength)
                    var totalRead = 0
                    while (totalRead < header.payloadLength) {
                        val read = inputStream?.read(payload, totalRead, header.payloadLength - totalRead) ?: -1
                        if (read == -1) break
                        totalRead += read
                    }

                    if (header.packetType == PacketHeader.TYPE_CAMERA_CONTROL) {
                        val jsonString = String(payload, Charsets.UTF_8)
                        val jsonObject = gson.fromJson(jsonString, JsonObject::class.java)
                        onCameraControlReceived(jsonObject)
                    }
                } catch (e: Exception) {
                    break
                }
            }
            disconnect()
        }
    }

    fun sendVideoFrame(frameData: ByteArray, isKeyframe: Boolean, timestamp: Long) {
        if (!isRunning) return
        scope.launch {
            try {
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_VIDEO_FRAME,
                    flags = if (isKeyframe) PacketHeader.FLAG_KEYFRAME else 0,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = timestamp,
                    payloadLength = frameData.size
                )
                val packet = Packet(header, frameData)
                outputStream?.write(packet.toByteArray())
                outputStream?.flush()
            } catch (e: Exception) {
                disconnect()
            }
        }
    }

    fun sendInstantCapture(metaJson: String, fileBytes: ByteArray) {
        if (!isRunning) return
        scope.launch {
            try {
                val metaBytes = metaJson.toByteArray(Charsets.UTF_8)
                val metaLenBytes = ByteBuffer.allocate(4).putInt(metaBytes.size).array()

                val payload = ByteArray(4 + metaBytes.size + fileBytes.size)
                System.arraycopy(metaLenBytes, 0, payload, 0, 4)
                System.arraycopy(metaBytes, 0, payload, 4, metaBytes.size)
                System.arraycopy(fileBytes, 0, payload, 4 + metaBytes.size, fileBytes.size)

                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_INSTANT_CAPTURE_CHUNK,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = System.currentTimeMillis(),
                    payloadLength = payload.size
                )
                val packet = Packet(header, payload)
                outputStream?.write(packet.toByteArray())
                outputStream?.flush()
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    fun disconnect() {
        isRunning = false
        try {
            socket?.close()
        } catch (ignored: Exception) {}
        onStatusChange(ConnectionStatus.DISCONNECTED)
    }
}
