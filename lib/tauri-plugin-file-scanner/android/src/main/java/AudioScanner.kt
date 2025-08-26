
package in.kieran.filescanner

import android.content.Context
import android.media.MediaScannerConnection
import android.os.Build
import android.provider.MediaStore
import android.util.Log
import androidx.annotation.RequiresApi
import in.kieran.filescanner.utils.Album
import in.kieran.filescanner.utils.Artist
import in.kieran.filescanner.utils.Genre
import in.kieran.filescanner.utils.Song
import getUriFromID
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume

class AudioScanner {
    private val TAG = "file-scanner"

    suspend fun readDirectory(
        mContext: Context, scanPath: String = "/storage/emulated/0"
    ): List<Song> {
        val scanDone = scanFileSuspend(mContext, scanPath)
        if (!scanDone) {
            Log.e(TAG, "readDirectory: scan failed or canceled")
            return emptyList()
        }

        return queryMediaStore(mContext)
    }

    private suspend fun scanFileSuspend(context: Context, path: String): Boolean {
        return suspendCancellableCoroutine { cont ->
            MediaScannerConnection.scanFile(context, arrayOf(path), null) { _, uri ->
                Log.d(TAG, "Media scan completed: $uri")
                cont.resume(uri != null)
            }
        }
    }

    private fun queryMediaStore(context: Context): List<Song> {
        val songList = mutableListOf<Song>()
        val proj = arrayListOf(
            MediaStore.Audio.Media._ID,
            MediaStore.Audio.Media.TITLE,
            MediaStore.Audio.Media.DISPLAY_NAME,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM_ID,
            MediaStore.Audio.Media.ARTIST_ID,
            MediaStore.Audio.Media.DURATION,
            MediaStore.Audio.Media.IS_MUSIC,
            MediaStore.Audio.Media.DATE_MODIFIED
        )

        if (Build.VERSION.SDK_INT >= 30) {
            proj.add(MediaStore.Audio.Media.GENRE)
            proj.add(MediaStore.Audio.Media.GENRE_ID)
        }

        context.contentResolver.query(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
            proj.toTypedArray(),
            null,
            null,
            MediaStore.Audio.Media.DEFAULT_SORT_ORDER
        )?.use { cursor ->
                while (cursor.moveToNext()) {
                    val isMusic = cursor.getInt(
                        cursor.getColumnIndexOrThrow(
                            MediaStore.Audio.Media.IS_MUSIC
                        )
                    )
                    Log.d(TAG, "queryMediaStore: File is music $isMusic")
                    if (isMusic != 0) {
                        try {
                            val id = cursor.getLong(
                                cursor.getColumnIndexOrThrow(
                                    MediaStore.Audio.Media._ID
                                )
                            )
                            val titleIndex =
                                if (cursor.getColumnIndex(MediaStore.Audio.Media.TITLE) != -1) cursor.getColumnIndex(
                                    MediaStore.Audio.Media.TITLE
                                )
                                else cursor.getColumnIndex(
                                    MediaStore.Audio.Media.DISPLAY_NAME
                                )

                            val song = Song(
                                title = cursor.getString(titleIndex),
                                duration = cursor.getLong(
                                    cursor.getColumnIndexOrThrow(
                                        MediaStore.Audio.Media.DURATION
                                    )
                                ) / 1000,
                                path = id.toString(),
                                artist = getArtist(cursor),
                                album = getAlbum(context, id, cursor),
                                genre = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                                    getGenre(cursor)
                                } else {
                                    null
                                },
                                playbackUrl = id.toString(),
                                song_coverPath_high = getUriFromID(context, id),
                                song_coverPath_low = getUriFromID(context, id),
                                type = "LOCAL"
                            )
                            songList.add(song)
                        } catch (e: Exception) {
                            Log.e(TAG, "queryMediaStore: error parsing song", e)
                        }
                    }
                }
            }

        return songList
    }

    private fun getArtist(cursor: android.database.Cursor): List<Artist>? {
        val artistId =
            cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ARTIST_ID))
        val artistName =
            cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ARTIST))
        return if (artistId != 0L) listOf(Artist(artistName, null)) else null
    }

    private fun getAlbum(context: Context, id: Long, cursor: android.database.Cursor): Album? {
        val albumId = cursor.getLong(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM_ID))
        val albumName = cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM))
        return if (albumId != 0L) Album(
            albumName,
            getUriFromID(context, id),
            getUriFromID(context, id)
        )
        else null
    }

    @RequiresApi(Build.VERSION_CODES.R)
    private fun getGenre(cursor: android.database.Cursor): List<Genre>? {
        val genreIdIndex = cursor.getColumnIndex(MediaStore.Audio.Media.GENRE_ID)
        if (genreIdIndex >= 0) {
            val genreId = cursor.getLong(genreIdIndex)
            val genreName =
                cursor.getString(cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.GENRE))
            if (genreId != 0L) return listOf(Genre(genreName))
        }
        return null
    }
}
