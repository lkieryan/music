package app.kieran.audioplayer.services.players

interface PlayerListeners {
    fun onTrackEnded(key: String)
    fun onTimeChange(key: String, time: Int)
}