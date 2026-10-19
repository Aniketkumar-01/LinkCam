package com.linkcam.app.scanner

import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.pdf.PdfDocument
import java.io.ByteArrayOutputStream

object PdfGenerator {
    /**
     * Converts a list of scanned page bitmaps into a valid, optimized PDF document byte array
     */
    fun createPdfFromBitmaps(bitmaps: List<Bitmap>): ByteArray {
        val document = PdfDocument()

        for ((index, bitmap) in bitmaps.withIndex()) {
            val pageInfo = PdfDocument.PageInfo.Builder(bitmap.width, bitmap.height, index + 1).create()
            val page = document.startPage(pageInfo)

            val canvas: Canvas = page.canvas
            val paint = Paint()
            canvas.drawBitmap(bitmap, 0f, 0f, paint)

            document.finishPage(page)
        }

        val outputStream = ByteArrayOutputStream()
        document.writeTo(outputStream)
        document.close()

        return outputStream.toByteArray()
    }
}
