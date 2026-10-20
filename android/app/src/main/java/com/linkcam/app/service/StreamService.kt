package com.linkcam.app.service

import android.app.*
import android.content.Context
import android.content.Intent
import android.net.wifi.WifiManager
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import com.linkcam.app.ui.MainActivity

class StreamService : Service() {

    companion object {
        private const val TAG = "StreamService"
        const val CHANNEL_ID = "linkcam_stream_channel"
        const val NOTIFICATION_ID = 1001
        const val ACTION_STOP = "com.linkcam.app.STOP_STREAM"

        fun startService(context: Context) {
            val intent = Intent(context, StreamService::class.java)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stopService(context: Context) {
            val intent = Intent(context, StreamService::class.java)
            context.stopService(intent)
        }
    }

    private var wakeLock: PowerManager.WakeLock? = null
    private var wifiLock: WifiManager.WifiLock? = null
    private var thermalListener: Any? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()

        // 1. Partial WakeLock to prevent CPU sleeping when screen dims/turns off
        val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = powerManager.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "LinkCam:StreamWakeLock").apply {
            acquire(10 * 60 * 1000L /* 10 hours max */)
        }

        // 2. High-performance Wi-Fi Lock to eliminate Wi-Fi power-save latency spikes
        val wifiManager = applicationContext.getSystemService(Context.WIFI_SERVICE) as WifiManager
        wifiLock = wifiManager.createWifiLock(WifiManager.WIFI_MODE_FULL_HIGH_PERF, "LinkCam:WifiLock").apply {
            setReferenceCounted(false)
            acquire()
        }

        // 3. Thermal monitoring listener (Android 10+)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            val listener = PowerManager.OnThermalStatusChangedListener { status ->
                Log.i(TAG, "Thermal status changed: $status")
            }
            thermalListener = listener
            try {
                powerManager.addThermalStatusListener(listener)
            } catch (ignored: Exception) {}
        }

        startForeground(NOTIFICATION_ID, buildNotification())
        Log.i(TAG, "StreamService started with WakeLock and HighPerf WifiLock")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopSelf()
            return START_NOT_STICKY
        }
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        wakeLock?.let {
            if (it.isHeld) it.release()
        }
        wifiLock?.let {
            if (it.isHeld) it.release()
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q && thermalListener != null) {
            val powerManager = getSystemService(Context.POWER_SERVICE) as PowerManager
            try {
                powerManager.removeThermalStatusListener(thermalListener as PowerManager.OnThermalStatusChangedListener)
            } catch (ignored: Exception) {}
        }
        super.onDestroy()
        Log.i(TAG, "StreamService destroyed, locks released")
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "LinkCam Streaming Service",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Keeps camera feed streaming to your Windows PC"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(): Notification {
        val launchIntent = Intent(this, MainActivity::class.java)
        val pendingIntent = PendingIntent.getActivity(
            this, 0, launchIntent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("LinkCam Active")
            .setContentText("Hardware 1080p60 stream active to Windows PC")
            .setSmallIcon(android.R.drawable.ic_menu_camera)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }
}
