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
    private val onStatusChange: (ConnectionStatus, String?) -> Unit,
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
                onStatusChange(ConnectionStatus.CONNECTING, null)
                val sock = Socket()
                sock.connect(InetSocketAddress(host, port), 5000)
                socket = sock
                outputStream = sock.getOutputStream()
                inputStream = sock.getInputStream()

                // Authenticate
                onStatusChange(ConnectionStatus.AUTHENTICATING, null)
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
                val inStream = inputStream ?: throw java.io.IOException("Socket stream closed")
                val respHeaderBytes = ByteArray(PacketHeader.HEADER_SIZE)
                if (!readFully(inStream, respHeaderBytes)) {
                    disconnect("Server closed connection during handshake")
                    return@launch
                }
                val respHeader = PacketHeader.readFrom(ByteBuffer.wrap(respHeaderBytes))

                if (respHeader.packetType == PacketHeader.TYPE_PAIRING_AUTH_RESPONSE) {
                    if (respHeader.payloadLength < 0 || respHeader.payloadLength > PacketHeader.MAX_PAYLOAD_SIZE) {
                        disconnect("Invalid authentication response length from server")
                        return@launch
                    }
                    val respPayload = ByteArray(respHeader.payloadLength)
                    if (!readFully(inStream, respPayload)) {
                        disconnect("Incomplete authentication payload received")
                        return@launch
                    }
                    val respText = String(respPayload, Charsets.UTF_8)
                    if (respText == "OK") {
                        isRunning = true
                        onStatusChange(ConnectionStatus.CONNECTED, null)
                        startReceiveLoop()
                    } else {
                        disconnect("Auth failed: $respText")
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
                val msg = when (e) {
                    is java.net.ConnectException -> "Connection refused ($host:$port). Verify LinkCam Desktop is open and port is forwarded."
                    is java.net.SocketTimeoutException -> "Timed out connecting to $host:$port. Ensure devices are on same network."
                    else -> e.localizedMessage ?: "Connection error ($host:$port)"
                }
                disconnect(msg)
            }
        }
    }

    private fun readFully(stream: InputStream, buffer: ByteArray, offset: Int = 0, length: Int = buffer.size): Boolean {
        var total = 0
        while (total < length) {
            val read = stream.read(buffer, offset + total, length - total)
            if (read == -1) return false
            total += read
        }
        return true
    }

    private fun startReceiveLoop() {
        scope.launch {
            val inStream = inputStream ?: return@launch
            val headerBuffer = ByteArray(PacketHeader.HEADER_SIZE)
            while (isRunning) {
                try {
                    if (!readFully(inStream, headerBuffer)) break

                    val header = PacketHeader.readFrom(ByteBuffer.wrap(headerBuffer))
                    if (header.payloadLength < 0 || header.payloadLength > PacketHeader.MAX_PAYLOAD_SIZE) break

                    val payload = ByteArray(header.payloadLength)
                    if (!readFully(inStream, payload)) break

                    when (header.packetType) {
                        PacketHeader.TYPE_CAMERA_CONTROL -> {
                            val jsonString = String(payload, Charsets.UTF_8)
                            val jsonObject = gson.fromJson(jsonString, JsonObject::class.java)
                            onCameraControlReceived(jsonObject)
                        }
                        PacketHeader.TYPE_KEEP_ALIVE_PING -> {
                            // Respond to keep-alive ping with acknowledgment
                            sendKeepAlivePing()
                        }
                        PacketHeader.TYPE_DISCONNECT -> {
                            disconnect()
                            break
                        }
                    }
                } catch (e: Exception) {
                    break
                }
            }
            disconnect()
        }
    }

    fun sendKeepAlivePing() {
        if (!isRunning) return
        scope.launch {
            try {
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_KEEP_ALIVE_PING,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = System.currentTimeMillis(),
                    payloadLength = 0
                )
                outputStream?.write(header.toByteArray())
                outputStream?.flush()
            } catch (ignored: Exception) {}
        }
    }

    fun sendCodecConfig(configJson: String) {
        if (!isRunning) return
        scope.launch {
            try {
                val payload = configJson.toByteArray(Charsets.UTF_8)
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_CODEC_CONFIG,
                    flags = PacketHeader.FLAG_CONFIG,
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

    fun sendTelemetry(
        fps: Float,
        batteryPct: Int,
        isCharging: Boolean,
        latencyMs: Int,
        connectionMode: String = "WiFi"
    ) {
        if (!isRunning) return
        scope.launch {
            try {
                val mode = when {
                    connectionMode.contains("USB", ignoreCase = true) -> "Usb"
                    connectionMode.contains("Hotspot", ignoreCase = true) -> "DirectHotspot"
                    else -> "WiFi"
                }
                val telemetryJson = """{"fps":$fps,"bitrate_kbps":4800,"latency_ms":$latencyMs,"battery_pct":$batteryPct,"is_charging":$isCharging,"thermal_state":"Normal","connection_mode":"$mode"}"""
                val payload = telemetryJson.toByteArray(Charsets.UTF_8)
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_TELEMETRY_HEARTBEAT,
                    sequence = sequenceCounter.getAndIncrement(),
                    timestamp = System.currentTimeMillis(),
                    payloadLength = payload.size
                )
                val packet = Packet(header, payload)
                outputStream?.write(packet.toByteArray())
                outputStream?.flush()
            } catch (ignored: Exception) {}
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

    fun disconnect(reason: String? = null) {
        isRunning = false
        try {
            socket?.close()
        } catch (ignored: Exception) {}
        onStatusChange(ConnectionStatus.DISCONNECTED, reason)
    }
}
