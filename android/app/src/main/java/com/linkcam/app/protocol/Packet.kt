package com.linkcam.app.protocol

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * LinkCam Wire Protocol Binary Packet (24-byte header)
 */
data class PacketHeader(
    val version: Byte = 1,
    val packetType: Byte,
    var flags: Byte = 0,
    val sequence: Int,
    val timestamp: Long,
    val payloadLength: Int
) {
    companion object {
        val MAGIC_BYTES = byteArrayOf('L'.code.toByte(), 'N'.code.toByte(), 'K'.code.toByte(), 'C'.code.toByte())
        const val HEADER_SIZE = 24

        const val TYPE_KEEP_ALIVE_PING: Byte = 0x00
        const val TYPE_VIDEO_FRAME: Byte = 0x01
        const val TYPE_AUDIO_FRAME: Byte = 0x02
        const val TYPE_INSTANT_CAPTURE_CHUNK: Byte = 0x03
        const val TYPE_CODEC_CONFIG: Byte = 0x04
        const val TYPE_KEYFRAME_REQUEST: Byte = 0x05
        const val TYPE_DISCONNECT: Byte = 0x06
        const val TYPE_CHUNK_RETRANSMIT: Byte = 0x07
        const val TYPE_PAIRING_AUTH: Byte = 0x10
        const val TYPE_PAIRING_AUTH_RESPONSE: Byte = 0x11
        const val TYPE_PAIRING_VERIFY: Byte = 0x12
        const val TYPE_CAMERA_CONTROL: Byte = 0x20
        const val TYPE_CAMERA_CONTROL_ACK: Byte = 0x21
        const val TYPE_TELEMETRY_HEARTBEAT: Byte = 0x30

        const val FLAG_KEYFRAME: Byte = 0x01
        const val FLAG_CONFIG: Byte = 0x02
        const val FLAG_END_OF_STREAM: Byte = 0x04
        const val FLAG_ENCRYPTED: Byte = 0x08

        const val MAX_PAYLOAD_SIZE = 10 * 1024 * 1024 // 10 MB max capture chunk (DoS guard)
        const val MAX_VIDEO_FRAME_SIZE = 2 * 1024 * 1024 // 2 MB max video frame

        fun readFrom(buffer: ByteBuffer): PacketHeader {
            buffer.order(ByteOrder.BIG_ENDIAN)
            val magic = ByteArray(4)
            buffer.get(magic)
            if (!magic.contentEquals(MAGIC_BYTES)) {
                throw IllegalArgumentException("Invalid magic header")
            }

            val version = buffer.get()
            val packetType = buffer.get()
            val flags = buffer.get()
            val reserved = buffer.get()
            val sequence = buffer.int
            val timestamp = buffer.long
            val payloadLength = buffer.int

            if (payloadLength < 0 || payloadLength > MAX_PAYLOAD_SIZE) {
                throw IllegalArgumentException("Payload length out of bounds: $payloadLength (max: $MAX_PAYLOAD_SIZE)")
            }

            return PacketHeader(version, packetType, flags, sequence, timestamp, payloadLength)
        }
    }

    fun toByteArray(): ByteArray {
        val buffer = ByteBuffer.allocate(HEADER_SIZE)
        buffer.order(ByteOrder.BIG_ENDIAN)
        buffer.put(MAGIC_BYTES)
        buffer.put(version)
        buffer.put(packetType)
        buffer.put(flags)
        buffer.put(0.toByte()) // reserved
        buffer.putInt(sequence)
        buffer.putLong(timestamp)
        buffer.putInt(payloadLength)
        return buffer.array()
    }
}

data class Packet(
    val header: PacketHeader,
    val payload: ByteArray
) {
    fun toByteArray(): ByteArray {
        val bytes = ByteArray(PacketHeader.HEADER_SIZE + payload.size)
        System.arraycopy(header.toByteArray(), 0, bytes, 0, PacketHeader.HEADER_SIZE)
        System.arraycopy(payload, 0, bytes, PacketHeader.HEADER_SIZE, payload.size)
        return bytes
    }
}
