package app.kieran.audioplayer.services.interfaces

import app.kieran.audioplayer.models.MetadataArgs
import app.kieran.audioplayer.models.PlaybackState
import app.kieran.audioplayer.models.Track

interface MediaControls {
    fun play(key: String)
    fun pause(key: String)
    fun stop(key: String)

    fun seek(key: String, time: Int)

    fun load(key: String, src: String, autoplay: Boolean)

    fun updateMetadata(metadata: MetadataArgs?)
    fun updatePlayerState(isPlaying: Boolean, pos: Int)
}