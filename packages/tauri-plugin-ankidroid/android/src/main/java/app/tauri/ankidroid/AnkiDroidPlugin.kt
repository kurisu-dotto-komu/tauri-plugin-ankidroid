package app.tauri.ankidroid

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import app.tauri.plugin.JSObject

@TauriPlugin
class AnkiDroidPlugin(private val activity: Activity): Plugin(activity) {
    
    @Command
    fun hello(invoke: Invoke) {
        val args = invoke.parseArgs(HelloArgs::class.java)
        val ret = JSObject()
        ret.put("value", "Hello, ${args.name} from AnkiDroid plugin on Android!")
        invoke.resolve(ret)
    }
}

data class HelloArgs(val name: String)