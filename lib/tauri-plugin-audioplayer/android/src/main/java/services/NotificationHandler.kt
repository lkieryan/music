
package in.kieran.audioplayer.services

import android.app.Activity
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.media.session.MediaButtonReceiver
import in.kieran.audioplayer.AudioPlayerPlugin
import in.kieran.audioplayer.R
import in.kieran.audioplayer.services.Constants.NOTIFICATION_CHANNEL_ID
import in.kieran.audioplayer.services.Constants.NOTIFICATION_ID

class NotificationHandler (
    private val mContext: Context,
    private val token: MediaSessionCompat.Token,
    private val launcherIcon: Int,
) {
    private val notificationManager: NotificationManager =
        mContext.applicationContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    private var notificationBuilder: NotificationCompat.Builder =
        NotificationCompat.Builder(mContext, NOTIFICATION_CHANNEL_ID)

    var notification: Notification? = null

    init {
        createNotification()
        // Cancel all notifications
        notificationManager.cancelAll()
        createNotificationChannel()
    }

    private fun createNotificationChannel() {
        val existingChannel = notificationManager.getNotificationChannel(NOTIFICATION_CHANNEL_ID)
        if (existingChannel == null) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "Now playing",
                NotificationManager.IMPORTANCE_LOW
            )
            channel.enableLights(false)
            channel.enableVibration(false)
            channel.setShowBadge(false)

            notificationManager.createNotificationChannel(channel)
        }
    }

    private fun createNotification() {
        val mediaStyle = androidx.media.app.NotificationCompat.MediaStyle()
            .setMediaSession(token).setShowActionsInCompactView(0, 1, 2)

        val launchIntent = mContext.packageManager.getLaunchIntentForPackage(mContext.packageName)?.apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED
        }

        val clickIntent = PendingIntent
            .getActivity(
                mContext,
                0,
                launchIntent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
            )

        val prevIntent = MediaButtonReceiver.buildMediaButtonPendingIntent(
            mContext,
            PlaybackStateCompat.ACTION_SKIP_TO_PREVIOUS
        )

        val pauseIntent = MediaButtonReceiver.buildMediaButtonPendingIntent(
            mContext,
            PlaybackStateCompat.ACTION_PAUSE
        )

        val nextIntent = MediaButtonReceiver.buildMediaButtonPendingIntent(
            mContext,
            PlaybackStateCompat.ACTION_SKIP_TO_NEXT
        )

        notificationBuilder
            .setStyle(mediaStyle)
            .addAction(android.R.drawable.ic_media_previous, "Previous", prevIntent)
            .addAction(android.R.drawable.ic_media_pause, "Pause", pauseIntent)
            .addAction(android.R.drawable.ic_media_next, "Next", nextIntent)
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .setSmallIcon(launcherIcon)
            .setContentIntent(clickIntent)
            .setShowWhen(false)
            .build()

        notification = notificationBuilder.build()
    }

    fun clearNotification() {
        notificationManager.cancel(NOTIFICATION_ID)
    }

    fun updateMetadata() {
        if (notification == null) {
            createNotification()
        }
        notificationManager.notify(NOTIFICATION_ID, notification)
    }

    fun release() {
        notificationManager.cancelAll()
    }
}
