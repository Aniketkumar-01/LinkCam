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
import java.security.MessageDigest
import java.util.concurrent.atomic.AtomicInteger

internal object ChaCha20Cipher {
    private fun rotl(v: Int, c: Int): Int = (v shl c) or (v ushr (32 - c))

    private fun quarterRound(x: IntArray, a: Int, b: Int, c: Int, d: Int) {
        x[a] += x[b]
        x[d] = rotl(x[d] xor x[a], 16)
        x[c] += x[d]
        x[b] = rotl(x[b] xor x[c], 12)
        x[a] += x[b]
        x[d] = rotl(x[d] xor x[a], 8)
        x[c] += x[d]
        x[b] = rotl(x[b] xor x[c], 7)
    }

    fun crypt(data: ByteArray, key: ByteArray, nonce: ByteArray, initialCounter: Int = 1): ByteArray {
        val output = ByteArray(data.size)
        val state = IntArray(16)
        var counter = initialCounter

        val c0 = 0x61707865
        val c1 = 0x3320646e
        val c2 = 0x79622d32
        val c3 = 0x6b206574

        val k = IntArray(8)
        for (i in 0 until 8) {
            k[i] = (key[i * 4].toInt() and 0xFF) or
                    ((key[i * 4 + 1].toInt() and 0xFF) shl 8) or
                    ((key[i * 4 + 2].toInt() and 0xFF) shl 16) or
                    ((key[i * 4 + 3].toInt() and 0xFF) shl 24)
        }

        val n = IntArray(3)
        for (i in 0 until 3) {
            n[i] = (nonce[i * 4].toInt() and 0xFF) or
                    ((nonce[i * 4 + 1].toInt() and 0xFF) shl 8) or
                    ((nonce[i * 4 + 2].toInt() and 0xFF) shl 16) or
                    ((nonce[i * 4 + 3].toInt() and 0xFF) shl 24)
        }

        val working = IntArray(16)
        val block = ByteArray(64)
        var offset = 0

        while (offset < data.size) {
            state[0] = c0; state[1] = c1; state[2] = c2; state[3] = c3
            System.arraycopy(k, 0, state, 4, 8)
            state[12] = counter
            System.arraycopy(n, 0, state, 13, 3)
            System.arraycopy(state, 0, working, 0, 16)

            for (r in 0 until 10) {
                quarterRound(working, 0, 4, 8, 12)
                quarterRound(working, 1, 5, 9, 13)
                quarterRound(working, 2, 6, 10, 14)
                quarterRound(working, 3, 7, 11, 15)
                quarterRound(working, 0, 5, 10, 15)
                quarterRound(working, 1, 6, 11, 12)
                quarterRound(working, 2, 7, 8, 13)
                quarterRound(working, 3, 4, 9, 14)
            }

            for (i in 0 until 16) {
                val sum = working[i] + state[i]
                block[i * 4] = (sum and 0xFF).toByte()
                block[i * 4 + 1] = ((sum ushr 8) and 0xFF).toByte()
                block[i * 4 + 2] = ((sum ushr 16) and 0xFF).toByte()
                block[i * 4 + 3] = ((sum ushr 24) and 0xFF).toByte()
            }

            val chunkSize = Math.min(64, data.size - offset)
            for (i in 0 until chunkSize) {
                output[offset + i] = (data[offset + i].toInt() xor block[i].toInt()).toByte()
            }

            counter++
            offset += chunkSize
        }
        return output
    }
}

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
    private var sequenceCounter = AtomicInteger(1)
    private var isRunning = false
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val gson = Gson()
    private var sessionKey: ByteArray? = null

    private fun deriveSessionKey(token: String, sessionId: String): ByteArray {
        val md = MessageDigest.getInstance("SHA-256")
        return md.digest("$token:$sessionId:linkcam_secure_v1".toByteArray(Charsets.UTF_8))
    }

    private fun encryptPayload(payload: ByteArray, sequence: Int, timestamp: Long): ByteArray {
        val key = sessionKey ?: return payload
        return try {
            val nonce = ByteBuffer.allocate(12)
                .order(java.nio.ByteOrder.LITTLE_ENDIAN)
                .putInt(sequence)
                .putLong(timestamp)
                .array()
            ChaCha20Cipher.crypt(payload, key, nonce, initialCounter = 1)
        } catch (e: Exception) {
            payload
        }
    }

    fun connect(host: String, port: Int, token: String, sessionId: String, certFingerprint: String? = null) {
        sessionKey = deriveSessionKey(token, sessionId)
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
                val seq = sequenceCounter.getAndIncrement()
                val encrypted = encryptPayload(frameData, seq, timestamp)
                val baseFlag = if (isKeyframe) PacketHeader.FLAG_KEYFRAME.toInt() else 0
                val flag = (baseFlag or PacketHeader.FLAG_ENCRYPTED.toInt()).toByte()
                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_VIDEO_FRAME,
                    flags = flag,
                    sequence = seq,
                    timestamp = timestamp,
                    payloadLength = encrypted.size
                )
                val packet = Packet(header, encrypted)
                outputStream?.write(packet.toByteArray())
                outputStream?.flush()
            } catch (e: Exception) {
                disconnect()
            }
        }
    }

    fun sendAudioPacket(packet: Packet) {
        if (!isRunning) return
        scope.launch {
            try {
                val encrypted = encryptPayload(packet.payload, packet.header.sequence, packet.header.timestamp)
                val newFlags = (packet.header.flags.toInt() or PacketHeader.FLAG_ENCRYPTED.toInt()).toByte()
                val newHeader = PacketHeader(
                    packetType = packet.header.packetType,
                    flags = newFlags,
                    sequence = packet.header.sequence,
                    timestamp = packet.header.timestamp,
                    payloadLength = encrypted.size
                )
                val encPacket = Packet(newHeader, encrypted)
                outputStream?.write(encPacket.toByteArray())
                outputStream?.flush()
            } catch (ignored: Exception) {
                // Audio packet loss over lossy Wi-Fi should not sever the session
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

                val rawPayload = ByteArray(4 + metaBytes.size + fileBytes.size)
                System.arraycopy(metaLenBytes, 0, rawPayload, 0, 4)
                System.arraycopy(metaBytes, 0, rawPayload, 4, metaBytes.size)
                System.arraycopy(fileBytes, 0, rawPayload, 4 + metaBytes.size, fileBytes.size)

                val seq = sequenceCounter.getAndIncrement()
                val ts = System.currentTimeMillis()
                val encrypted = encryptPayload(rawPayload, seq, ts)

                val header = PacketHeader(
                    packetType = PacketHeader.TYPE_INSTANT_CAPTURE_CHUNK,
                    flags = PacketHeader.FLAG_ENCRYPTED,
                    sequence = seq,
                    timestamp = ts,
                    payloadLength = encrypted.size
                )
                val packet = Packet(header, encrypted)
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
