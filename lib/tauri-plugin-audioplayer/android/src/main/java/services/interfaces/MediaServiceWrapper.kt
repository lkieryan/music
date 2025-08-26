package app.kieran.audioplayer.services.interfaces

import android.support.v4.media.session.MediaSessionCompat
import app.kieran.audioplayer.models.PlaybackState
import app.kieran.audioplayer.services.NotificationHandler

interface MediaServiceWrapper {
    val controls: MediaControls

    fun decideQuit()

    fun setMainActivityStatus(isRunning: Boolean)

    fun addMediaPlayerCallbacks(callbacks: MediaPlayerCallbacks)
    fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback)
}