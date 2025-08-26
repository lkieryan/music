
package app.kieran.audioplayer.services

import android.content.Context
import android.util.Log
import app.kieran.audioplayer.models.Song
import app.kieran.audioplayer.services.players.GenericPlayer
import app.kieran.audioplayer.services.players.LocalPlayer
import app.kieran.audioplayer.services.players.PlayerListeners

class PlaybackManager(mContext: Context, private val playerListeners: PlayerListeners) {
    private val players: HashMap<String, GenericPlayer> = hashMapOf(Pair("LOCAL", LocalPlayer()))

    init {
        for (player in players.values) {
            player.setPlayerListeners(playerListeners)
        }
    }

    fun stop(key: String) {
        players[key]?.stop()
    }

    fun release() {
        players.forEach {
            it.value.release()
        }
    }

    fun pause(key: String) {
        players[key]?.pause()
    }

    fun play(key: String) {
        players[key]?.play()
    }

    fun seek(key: String, pos: Int) {
        players[key]?.progress = pos
    }

    fun canPlay(key: String, song: Song): Boolean {
        return players[key]?.canPlay(song) == true
    }

    fun load(key: String, context: Context, src: String, autoPlay: Boolean) {
        players[key]?.load(context, src, autoPlay)
    }
}
