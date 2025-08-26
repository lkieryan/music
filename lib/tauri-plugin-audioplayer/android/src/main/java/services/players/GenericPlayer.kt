package app.kieran.audioplayer.services.players

import android.content.Context
import app.kieran.audioplayer.models.Song

abstract class GenericPlayer {
    open var key: String = ""
    
    abstract var progress: Int
    abstract val isPlaying: Boolean
    
    abstract fun canPlay(song: Song): Boolean
    abstract fun load(mContext: Context, src: String, autoPlay: Boolean)
    abstract fun setPlayerListeners(playerListeners: PlayerListeners)
    abstract fun removePlayerListeners()
    abstract fun play()
    abstract fun pause()
    abstract fun stop()
    abstract fun release()
}