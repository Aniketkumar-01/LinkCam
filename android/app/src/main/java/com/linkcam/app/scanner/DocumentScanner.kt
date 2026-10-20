package com.linkcam.app.scanner

import android.graphics.*
import java.security.MessageDigest
import java.util.UUID

data class DocumentScanResult(
    val transferId: String,
    val fileName: String,
    val mimeType: String,
    val fileBytes: ByteArray,
    val sha256Checksum: String
)

class DocumentScanner {

    /**
     * Auto-crops and applies perspective transformation to detected document corners
     */
    fun processDocumentCrop(
        sourceBitmap: Bitmap,
        corners: List<PointF> // 4 corners: Top-Left, Top-Right, Bottom-Right, Bottom-Left
    ): Bitmap {
        if (corners.size != 4) return sourceBitmap

        // Calculate maximum width and height from corner coordinates
        val widthA = Math.hypot((corners[1].x - corners[0].x).toDouble(), (corners[1].y - corners[0].y).toDouble())
        val widthB = Math.hypot((corners[2].x - corners[3].x).toDouble(), (corners[2].y - corners[3].y).toDouble())
        val maxWidth = Math.max(widthA, widthB).toFloat()

        val heightA = Math.hypot((corners[3].x - corners[0].x).toDouble(), (corners[3].y - corners[0].y).toDouble())
        val heightB = Math.hypot((corners[2].x - corners[1].x).toDouble(), (corners[2].y - corners[1].y).toDouble())
        val maxHeight = Math.max(heightA, heightB).toFloat()

        val targetBitmap = Bitmap.createBitmap(maxWidth.toInt(), maxHeight.toInt(), Bitmap.Config.ARGB_8888)
        val canvas = Canvas(targetBitmap)

        val srcPoints = floatArrayOf(
            corners[0].x, corners[0].y,
            corners[1].x, corners[1].y,
            corners[2].x, corners[2].y,
            corners[3].x, corners[3].y
        )

        val dstPoints = floatArrayOf(
            0f, 0f,
            maxWidth, 0f,
            maxWidth, maxHeight,
            0f, maxHeight
        )

        val matrix = Matrix()
        matrix.setPolyToPoly(srcPoints, 0, dstPoints, 0, 4)

        val paint = Paint(Paint.ANTI_ALIAS_FLAG or Paint.FILTER_BITMAP_FLAG)
        canvas.drawBitmap(sourceBitmap, matrix, paint)

        return targetBitmap
    }

    /**
     * Packages a scanned document or photo into a ready-to-stream LinkCam InstantCapture payload
     */
    fun prepareScanPayload(
        bitmaps: List<Bitmap>,
        isPdf: Boolean,
        baseName: String = "Document_Scan"
    ): DocumentScanResult {
        val transferId = UUID.randomUUID().toString()
        val extension = if (isPdf) "pdf" else "jpg"
        val fileName = "${baseName}_${System.currentTimeMillis()}.$extension"
        val mimeType = if (isPdf) "application/pdf" else "image/jpeg"

        val fileBytes = if (isPdf) {
            PdfGenerator.createPdfFromBitmaps(bitmaps)
        } else {
            val out = java.io.ByteArrayOutputStream()
            bitmaps.first().compress(Bitmap.CompressFormat.JPEG, 92, out)
            out.toByteArray()
        }

        val digest = MessageDigest.getInstance("SHA-256")
        val hashBytes = digest.digest(fileBytes)
        val checksum = hashBytes.joinToString("") { "%02x".format(it) }

        return DocumentScanResult(
            transferId = transferId,
            fileName = fileName,
            mimeType = mimeType,
            fileBytes = fileBytes,
            sha256Checksum = checksum
        )
    }

    companion object {
        /**
         * Checks whether Google Play Services is available and operational on this device.
         * On Xiaomi HyperOS / China ROMs / de-Googled devices, this returns false, triggering
         * LinkCam's offline CameraX edge-crop and PDF generator fallback.
         */
        fun isGooglePlayServicesAvailable(context: android.content.Context): Boolean {
            return try {
                val availability = com.google.android.gms.common.GoogleApiAvailability.getInstance()
                val resultCode = availability.isGooglePlayServicesAvailable(context)
                resultCode == com.google.android.gms.common.ConnectionResult.SUCCESS
            } catch (e: Throwable) {
                false
            }
        }
    }
}
