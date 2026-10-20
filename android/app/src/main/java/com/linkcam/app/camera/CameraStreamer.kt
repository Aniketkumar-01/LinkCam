package com.linkcam.app.camera

import android.content.Context
import android.graphics.ImageFormat
import android.graphics.Rect
import android.graphics.YuvImage
import android.util.Size
import androidx.camera.core.*
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.core.content.ContextCompat
import androidx.lifecycle.LifecycleOwner
import java.io.ByteArrayOutputStream
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

enum class CameraLens {
    REAR_WIDE,
    REAR_ULTRA_WIDE,
    REAR_TELEPHOTO,
    FRONT
}

class CameraStreamer(
    private val context: Context,
    private val lifecycleOwner: LifecycleOwner,
    private val onCodecConfig: ((ByteArray) -> Unit)? = null,
    private val onFrameEncoded: (ByteArray, Boolean, Long) -> Unit
) {
    private var cameraProvider: ProcessCameraProvider? = null
    private var camera: Camera? = null
    private var preview: Preview? = null
    private var imageAnalysis: ImageAnalysis? = null
    private var surfaceProvider: Preview.SurfaceProvider? = null
    private val cameraExecutor: ExecutorService = Executors.newSingleThreadExecutor()
    private var h264Encoder: MediaCodecH264Encoder? = null

    private var currentLens: CameraLens = CameraLens.REAR_WIDE
    private var isTorchEnabled: Boolean = false
    private var targetResolution = Size(1920, 1080)
    private var frameSequence: Long = 0

    fun setSurfaceProvider(provider: Preview.SurfaceProvider?) {
        this.surfaceProvider = provider
        if (cameraProvider != null) {
            bindCameraUseCases()
        }
    }

    fun start() {
        val cameraProviderFuture = ProcessCameraProvider.getInstance(context)
        cameraProviderFuture.addListener({
            cameraProvider = cameraProviderFuture.get()
            bindCameraUseCases()
        }, ContextCompat.getMainExecutor(context))
    }

    private fun bindCameraUseCases() {
        val provider = cameraProvider ?: return
        provider.unbindAll()

        // Initialize or reconfigure hardware H.264 encoder
        h264Encoder?.release()
        h264Encoder = MediaCodecH264Encoder(
            width = targetResolution.width,
            height = targetResolution.height,
            bitrateBps = 8_000_000,
            fps = 60,
            onCodecConfig = { configBytes -> onCodecConfig?.invoke(configBytes) },
            onFrameEncoded = onFrameEncoded
        )

        val cameraSelector = when (currentLens) {
            CameraLens.FRONT -> CameraSelector.DEFAULT_FRONT_CAMERA
            else -> CameraSelector.DEFAULT_BACK_CAMERA
        }

        imageAnalysis = ImageAnalysis.Builder()
            .setTargetResolution(targetResolution)
            .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
            .setOutputImageFormat(ImageAnalysis.OUTPUT_IMAGE_FORMAT_YUV_420_888)
            .build()
            .also { analysis ->
                analysis.setAnalyzer(cameraExecutor) { imageProxy ->
                    processImageFrame(imageProxy)
                }
            }

        val useCases = mutableListOf<UseCase>()

        val sp = surfaceProvider
        if (sp != null) {
            val prev = Preview.Builder()
                .setTargetResolution(targetResolution)
                .build()
            prev.setSurfaceProvider(sp)
            preview = prev
            useCases.add(prev)
        }

        imageAnalysis?.let { useCases.add(it) }

        try {
            camera = provider.bindToLifecycle(
                lifecycleOwner,
                cameraSelector,
                *useCases.toTypedArray()
            )
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun processImageFrame(imageProxy: ImageProxy) {
        val timestamp = imageProxy.imageInfo.timestamp
        frameSequence++

        val nv21 = yuv420ToNv21(imageProxy)
        val encoder = h264Encoder
        if (encoder != null) {
            // Hardware-accelerated H.264 encoding path
            encoder.encodeFrame(nv21, timestamp)
        } else {
            // Fallback: CPU compressed JPEG stream frame
            val yuvImage = YuvImage(nv21, ImageFormat.NV21, imageProxy.width, imageProxy.height, intArrayOf(imageProxy.width, imageProxy.width))
            val out = ByteArrayOutputStream()
            yuvImage.compressToJpeg(Rect(0, 0, imageProxy.width, imageProxy.height), 85, out)

            val frameBytes = out.toByteArray()
            val isKeyframe = (frameSequence % 30L == 0L)
            onFrameEncoded(frameBytes, isKeyframe, timestamp / 1_000_000L)
        }

        imageProxy.close()
    }

    fun requestKeyframe() {
        h264Encoder?.requestKeyframe()
    }

    fun setBitrate(bitrateBps: Int) {
        h264Encoder?.setBitrate(bitrateBps)
    }

    fun setLens(lens: CameraLens) {
        if (currentLens != lens) {
            currentLens = lens
            bindCameraUseCases()
        }
    }

    fun setTorch(enabled: Boolean) {
        isTorchEnabled = enabled
        camera?.cameraControl?.enableTorch(enabled)
    }

    fun setZoom(ratio: Float) {
        camera?.cameraControl?.setZoomRatio(ratio)
    }

    fun setExposure(ev: Int) {
        camera?.cameraControl?.setExposureCompensationIndex(ev)
    }

    fun stop() {
        h264Encoder?.release()
        h264Encoder = null
        cameraProvider?.unbindAll()
        cameraExecutor.shutdown()
    }

    private fun yuv420ToNv21(image: ImageProxy): ByteArray {
        val width = image.width
        val height = image.height
        val ySize = width * height
        val uvSize = width * height / 2
        val nv21 = ByteArray(ySize + uvSize)

        val yBuffer = image.planes[0].buffer
        val uBuffer = image.planes[1].buffer
        val vBuffer = image.planes[2].buffer

        val yRowStride = image.planes[0].rowStride
        val yPixelStride = image.planes[0].pixelStride

        var pos = 0
        for (row in 0 until height) {
            yBuffer.position(row * yRowStride)
            for (col in 0 until width) {
                nv21[pos++] = yBuffer.get()
                if (yPixelStride > 1 && col < width - 1) {
                    yBuffer.position(yBuffer.position() + yPixelStride - 1)
                }
            }
        }

        val uvRowStride = image.planes[2].rowStride
        val uvPixelStride = image.planes[2].pixelStride

        for (row in 0 until height / 2) {
            for (col in 0 until width / 2) {
                val vIndex = row * uvRowStride + col * uvPixelStride
                val uIndex = row * image.planes[1].rowStride + col * image.planes[1].pixelStride
                nv21[pos++] = vBuffer.get(vIndex)
                nv21[pos++] = uBuffer.get(uIndex)
            }
        }

        return nv21
    }
}
