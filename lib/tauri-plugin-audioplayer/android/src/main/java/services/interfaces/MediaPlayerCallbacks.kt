package app.kieran.audioplayer.services.interfaces

interface MediaPlayerCallbacks {
    fun onPlay(key: String) {}
    fun onPause(key: String) {}
    fun onStop(key: String) {}
    fun onSongEnded(key: String) {}
    fun onTimeChange(key: String, time: Int) {}
}