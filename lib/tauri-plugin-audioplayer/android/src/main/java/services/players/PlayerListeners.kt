package app.kieran.audioplayer.services.players

interface PlayerListeners {
    fun onSongEnded(key: String)
    fun onTimeChange(key: String, time: Int)
}