package app.kieran.audioplayer.models

import java.io.Serializable

data class Artist( val artist_name: String, val artist_coverpath: Any?) : Serializable

data class Album(val album_name: String, val album_coverpath_high: String?, val album_coverpath_low: String?) : Serializable

data class Genre(val genre_name: String) : Serializable

data class Track(
    val title: String,
    val duration: Long,
    val path: String?,
    val artist: List<Artist>?,
    val album: Album?,
    val genre: List<Genre>?,
    val playbackUrl: String?,
    val track_coverPath_low: String?,
    val track_coverPath_high: String?,
) : Serializable

fun List<Artist>.toArtistString(): String {
    return joinToString(", ") {
        it.artist_name
    }
}
