package org.churchpresenter.presentationengine.fonts

import org.mozilla.javascript.Context
import org.mozilla.javascript.ContextFactory
import org.mozilla.javascript.Function
import org.mozilla.javascript.ScriptableObject
import org.mozilla.javascript.typedarrays.NativeUint8Array
import java.io.ByteArrayInputStream
import java.util.concurrent.TimeUnit

/** Adapts the bundled MTX decoder to the JVM without depending on an installed Office app. */
internal object CompressedEmbeddedFont {
    private const val MAX_FONT_BYTES = 16 * 1024 * 1024
    private const val INSTRUCTION_INTERVAL = 100_000
    private val timeBudget = TimeUnit.SECONDS.toNanos(15)

    private val contexts = object : ContextFactory() {
        override fun makeContext(): Context = super.makeContext().apply {
            languageVersion = Context.VERSION_ES6
            instructionObserverThreshold = INSTRUCTION_INTERVAL
            setClassShutter { false }
        }

        override fun observeInstructionCount(cx: Context, instructionCount: Int) {
            val started = cx.getThreadLocal(this) as Long
            check(System.nanoTime() - started < timeBudget) { "Embedded font decoding timed out" }
        }
    }

    private val decoder by lazy {
        contexts.call { cx ->
            checkNotNull(javaClass.getResourceAsStream("/mtx/decoder.js")).bufferedReader().use {
                cx.compileReader(it, "mtx/decoder.js", 1, null)
            }
        }
    }

    fun decode(eot: ByteArray): ByteArrayInputStream? = runCatching {
        val script = decoder
        contexts.call { cx ->
            cx.putThreadLocal(contexts, System.nanoTime())
            val scope = cx.initSafeStandardObjects()
            val exports = cx.newObject(scope)
            ScriptableObject.putProperty(scope, "exports", exports)
            script.exec(cx, scope)
            val options = cx.newObject(scope)
            ScriptableObject.putProperty(options, "maxOutputBytes", MAX_FONT_BYTES)
            val input = cx.newObject(scope, "Uint8Array", arrayOf(eot.size)) as NativeUint8Array
            eot.copyInto(input.buffer.buffer)
            val decode = ScriptableObject.getProperty(exports, "eotToTtf") as Function
            val font = decode.call(cx, scope, exports, arrayOf(input, options)) as NativeUint8Array
            ByteArrayInputStream(font.buffer.buffer, font.byteOffset, font.byteLength)
        }
    }.getOrNull()
}
