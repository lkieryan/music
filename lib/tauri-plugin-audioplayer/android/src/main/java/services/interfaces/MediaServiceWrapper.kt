
package in.kieran.audioplayer.services.interfaces

import android.support.v4.media.session.MediaSessionCompat
import in.kieran.audioplayer.models.PlaybackState
import in.kieran.audioplayer.services.NotificationHandler

interface MediaServiceWrapper {
    val controls: MediaControls

    fun decideQuit()

    fun setMainActivityStatus(isRunning: Boolean)

    fun addMediaPlayerCallbacks(callbacks: MediaPlayerCallbacks)
    fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback)
}
