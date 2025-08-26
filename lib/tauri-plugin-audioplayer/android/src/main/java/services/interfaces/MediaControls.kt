
package in.kieran.audioplayer.services.interfaces

import in.kieran.audioplayer.models.MetadataArgs
import in.kieran.audioplayer.models.PlaybackState
import in.kieran.audioplayer.models.Song

interface MediaControls {
    fun play(key: String)
    fun pause(key: String)
    fun stop(key: String)

    fun seek(key: String, time: Int)

    fun load(key: String, src: String, autoplay: Boolean)

    fun updateMetadata(metadata: MetadataArgs?)
    fun updatePlayerState(isPlaying: Boolean, pos: Int)
}
