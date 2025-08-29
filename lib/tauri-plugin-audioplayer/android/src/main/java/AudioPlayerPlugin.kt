
package in.kieran.audioplayer

import android.Manifest
import android.app.Activity
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import android.webkit.WebView
import in.kieran.audioplayer.models.MetadataArgs
import in.kieran.audioplayer.models.Song
import in.kieran.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
internal class LoadArgs {
    lateinit var key: String
    lateinit var src: String
    var autoplay: Boolean = false
}

@InvokeArg
internal class KeyArgs {
    lateinit var key: String
}

@InvokeArg
internal class SeekArgs {
    lateinit var key: String
    var seek = 0f
}

@InvokeArg
internal class UpdateMetadataArgs {
    lateinit var metadata: MetadataArgs
}

@InvokeArg
internal class UpdateStateArgs {
    var playing: Boolean = false
    var pos: Int = 0
}

@InvokeArg
class SetEventHandlerArgs {
    lateinit var handler: Channel
}

@InvokeArg
class TokenArgs {
    lateinit var token: String
}

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.READ_MEDIA_AUDIO, Manifest.permission.READ_MEDIA_IMAGES], alias = "readMedia")
    ]
)
class AudioPlayerPlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = AudioPlayerRemote(activity)
    private var channel: Channel? = null

    init {
        implementation.addMediaCallbacks(
            callbacks = object : MediaPlayerCallbacks {
                override fun onPlay(key: String) {
                    super.onPlay(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onPlay", ret)
                }

                override fun onPause(key: String) {
                    super.onPause(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onPause", ret)
                }

                override fun onSongEnded(key: String) {
                    super.onSongEnded(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onSongEnded", ret)
                }

                override fun onStop(key: String) {
                    super.onStop(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onStop", ret)
                }

                override fun onTimeChange(key: String, time: Int) {
                    super.onTimeChange(key, time)
                    val ret = JSObject()
                    ret.put("key", key);
                    ret.put("pos", time)
                    trigger("onTimeChange", ret)
                }
            }
        )

        implementation.addMediaSessionCallbacks(object : MediaSessionCompat.Callback() {
            override fun onPlay() {
                super.onPlay()
                val ret = JSObject()
                ret.put("event", "onPlay")
                this@AudioPlayerPlugin.channel?.send(ret)
            }

            override fun onPause() {
                super.onPause()
                val ret = JSObject()
                ret.put("event", "onPause")
                this@AudioPlayerPlugin.channel?.send(ret)
            }

            override fun onSeekTo(pos: Long) {
                super.onSeekTo(pos)
                val ret = JSObject()
                ret.put("event", "onSeekTo")
                ret.put("pos", pos)
                this@AudioPlayerPlugin.channel?.send(ret)
            }

            override fun onStop() {
                super.onStop()
                val ret = JSObject()
                ret.put("event", "onStop")
                this@AudioPlayerPlugin.channel?.send(ret)
            }

            override fun onSkipToNext() {
                val ret = JSObject()
                ret.put("event", "onSkipToNext")
                this@AudioPlayerPlugin.channel?.send(ret)
            }

            override fun onSkipToPrevious() {
                val ret = JSObject()
                ret.put("event", "onSkipToPrevious")
                this@AudioPlayerPlugin.channel?.send(ret)
            }
        })
    }

    @Command
    fun load(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(LoadArgs::class.java)
            implementation.controls?.load(args.key, args.src, args.autoplay)
        } catch (e: Exception) {
            Log.d("TAG", "load: failed to load audio $e")
        }
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun play(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.play(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun pause(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.pause(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun stop(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.stop(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun seek(invoke: Invoke) {
        val args = invoke.parseArgs(SeekArgs::class.java)
        implementation.controls?.seek(args.key, args.seek.toInt())
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun updateNotification(invoke: Invoke) {
        val args = invoke.parseArgs(UpdateMetadataArgs::class.java)
        implementation.controls?.updateMetadata(args.metadata)

        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun updateNotificationState(invoke: Invoke) {
        val args = invoke.parseArgs(UpdateStateArgs::class.java)
        implementation.controls?.updatePlayerState(args.playing, args.pos)

        val ret = JSObject()
        invoke.resolve(ret)
    }

    // This command should not be added to the `build.rs` and exposed as it is only
    // used internally from the rust backend.
    @Command
    fun setEventHandler(invoke: Invoke) {
        val args = invoke.parseArgs(SetEventHandlerArgs::class.java)
        this.channel = args.handler
        invoke.resolve()
    }
}
