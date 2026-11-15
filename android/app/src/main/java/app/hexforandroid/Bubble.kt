package app.hexforandroid

import android.animation.ValueAnimator
import android.annotation.SuppressLint
import android.content.Context
import android.graphics.PixelFormat
import android.os.SystemClock
import android.view.Gravity
import android.view.HapticFeedbackConstants
import android.view.MotionEvent
import android.view.View
import android.view.ViewConfiguration
import android.view.WindowManager
import android.view.animation.OvershootInterpolator
import android.widget.Toast
import kotlin.math.hypot
import kotlin.math.max

/**
 * The floating dictation bubble, an accessibility overlay showing [DotScope].
 *
 * - Tap: start recording; tap again to stop, transcribe and insert.
 * - Hold: record while held; lift to stop and insert.
 * - Long-press while a tapped recording runs: cancel it.
 * - Drag: move the bubble; it snaps to the nearer screen edge.
 */
class Bubble(
    private val context: Context,
    private val controls: HexAccessibilityService.Controls,
) {
    private val windows = context.getSystemService(WindowManager::class.java)
    private val settings = Settings(context)
    private val scope = DotScope(context)
    private var shown = false
    private var lastMessage: String? = null
    private var targetWidth = DotScope.REST_WIDTH
    private var resize: ValueAnimator? = null

    private val params = WindowManager.LayoutParams(
        0, 0,
        WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
        WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
        PixelFormat.TRANSLUCENT,
    ).apply {
        gravity = Gravity.TOP or Gravity.START
        title = "Hex"
    }

    init {
        scope.setOnTouchListener(Gestures())
        // TalkBack's activate gesture arrives as a click: toggle recording.
        scope.setOnClickListener {
            if (Dictation.recording) controls.finish() else controls.press()
        }
        scope.contentDescription = "Hex dictation. Tap to start or stop, hold to talk."
        restyle()
    }

    /** Applies the shape and size from settings. */
    fun restyle() {
        scope.shape = BubbleShape.of(settings.bubbleShape)
        scope.size = BubbleSize.of(settings.bubbleSize)
        scope.pillWidth = DotScope.REST_WIDTH
        targetWidth = DotScope.REST_WIDTH
        resize()
        if (shown) place()
    }

    fun show() {
        if (shown) return
        resize()
        place()
        windows.addView(scope, params)
        shown = true
    }

    fun hide() {
        if (!shown) return
        resize?.cancel()
        windows.removeViewImmediate(scope)
        shown = false
    }

    /** The screen changed: folded or unfolded, rotated, or switched between light and dark. */
    fun keepOnScreen() {
        scope.applyTheme(Tokens.of(context).dark)
        if (shown) place()
    }

    fun render(state: Dictation.State) {
        scope.update(state)
        if (scope.shape == BubbleShape.Pill) {
            val target = when (state.phase) {
                Dictation.Phase.Recording -> DotScope.RECORDING_WIDTH
                Dictation.Phase.Transcribing -> DotScope.PROCESSING_WIDTH
                else -> DotScope.REST_WIDTH
            }
            if (target != targetWidth) animatePill(target)
        }
        val message = state.message
        if (message != null && message != lastMessage && shown) {
            Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
        }
        lastMessage = message
    }

    private fun resize() {
        val (w, h) = scope.viewPx()
        params.width = w
        params.height = h
    }

    /** Grows or shrinks the pill, keeping the edge it is docked to still. */
    private fun animatePill(target: Float) {
        targetWidth = target
        resize?.cancel()
        resize = ValueAnimator.ofFloat(scope.pillWidth, target).apply {
            duration = 260
            interpolator = OvershootInterpolator(1.1f)
            addUpdateListener {
                scope.pillWidth = it.animatedValue as Float
                place()
            }
            start()
        }
    }

    /** 0..1: how far a hold has lifted and enlarged the bubble. */
    private var lift = 0f
    private var liftAnimation: ValueAnimator? = null

    /**
     * While a press holds the bubble to talk, it grows and rises above the
     * finger so the scope stays in view; it settles back on release.
     */
    private fun hold(on: Boolean) {
        liftAnimation?.cancel()
        liftAnimation = ValueAnimator.ofFloat(lift, if (on) 1f else 0f).apply {
            duration = if (on) 180 else 140
            interpolator = if (on) OvershootInterpolator(1.2f) else null
            addUpdateListener {
                lift = it.animatedValue as Float
                scope.magnify = 1f + HOLD_GROWTH * lift
                place()
            }
            start()
        }
    }

    private fun screen(): Pair<Int, Int> {
        val bounds = windows.currentWindowMetrics.bounds
        return bounds.width() to bounds.height()
    }

    /**
     * Sizes the window to the bubble and puts it on its saved edge and height
     * for the current screen. A hold lifts it: at full lift its bottom sits a
     * quarter of the way down its resting place, above the finger.
     */
    private fun place() {
        val (width, height) = screen()
        val resting = restingHeight()
        resize()
        params.x = if (settings.bubbleRight) width - params.width else 0
        val top = settings.bubbleTop * height
        params.y = (top - lift * (params.height - resting * 0.25f)).toInt()
        clamp()
        if (shown) windows.updateViewLayout(scope, params)
    }

    /** The window's height without a hold's magnification. */
    private fun restingHeight(): Float {
        val (_, plate) = scope.plateDp()
        val density = context.resources.displayMetrics.density
        return (plate / scope.magnify + 2 * DotScope.PAD) * density
    }

    private fun clamp() {
        val (width, height) = screen()
        params.x = params.x.coerceIn(0, max(0, width - params.width))
        params.y = params.y.coerceIn(0, max(0, height - params.height))
    }

    private fun snap() {
        val (width, height) = screen()
        settings.bubbleRight = params.x + params.width / 2 > width / 2
        settings.bubbleTop = params.y.toFloat() / height
        place()
    }

    private inner class Gestures : View.OnTouchListener {
        private val slop = ViewConfiguration.get(context).scaledTouchSlop
        private var downAt = 0L
        private var downX = 0f
        private var downY = 0f
        private var startX = 0
        private var startY = 0
        private var dragging = false
        /** This press started the recording (tap-to-lock or hold-to-talk). */
        private var startedHere = false
        /** A tapped recording keeps running after the finger lifts. */
        private var locked = false
        private var consumed = false
        /** The press became hold-to-talk: the bubble is lifted, and moving the finger doesn't drag it. */
        private var holding = false

        private val holdStart = Runnable {
            if (!dragging && startedHere && Dictation.recording) {
                holding = true
                hold(true)
            }
        }

        private val longPress = Runnable {
            if (!dragging && !startedHere && locked && Dictation.recording) {
                consumed = true
                locked = false
                controls.cancel()
                scope.performHapticFeedback(HapticFeedbackConstants.REJECT)
            }
        }

        @SuppressLint("ClickableViewAccessibility")
        override fun onTouch(v: View, event: MotionEvent): Boolean {
            when (event.actionMasked) {
                MotionEvent.ACTION_DOWN -> {
                    downAt = SystemClock.uptimeMillis()
                    downX = event.rawX
                    downY = event.rawY
                    startX = params.x
                    startY = params.y
                    dragging = false
                    consumed = false
                    startedHere = false
                    holding = false
                    if (!Dictation.recording && Dictation.state.phase != Dictation.Phase.Transcribing) {
                        startedHere = controls.press()
                        locked = false
                        if (startedHere) v.performHapticFeedback(HapticFeedbackConstants.CONFIRM)
                    }
                    v.postDelayed(longPress, CANCEL_MS)
                    v.postDelayed(holdStart, HOLD_MS)
                }
                MotionEvent.ACTION_MOVE -> {
                    val dx = event.rawX - downX
                    val dy = event.rawY - downY
                    if (holding) return true
                    if (!dragging && hypot(dx, dy) > slop) {
                        dragging = true
                        v.removeCallbacks(longPress)
                        v.removeCallbacks(holdStart)
                        if (startedHere) {
                            // The press was the start of a drag, not a dictation.
                            controls.cancel()
                            startedHere = false
                        }
                    }
                    if (dragging) {
                        params.x = startX + dx.toInt()
                        params.y = startY + dy.toInt()
                        clamp()
                        windows.updateViewLayout(scope, params)
                    }
                }
                MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                    v.removeCallbacks(longPress)
                    v.removeCallbacks(holdStart)
                    if (holding) {
                        holding = false
                        hold(false)
                    }
                    if (dragging) {
                        snap()
                        return true
                    }
                    if (consumed || event.actionMasked == MotionEvent.ACTION_CANCEL) return true
                    val held = SystemClock.uptimeMillis() - downAt
                    when {
                        startedHere && held >= HOLD_MS -> {
                            controls.finish()
                            v.performHapticFeedback(HapticFeedbackConstants.CONFIRM)
                        }
                        startedHere -> locked = true
                        locked && Dictation.recording -> {
                            locked = false
                            controls.finish()
                            v.performHapticFeedback(HapticFeedbackConstants.CONFIRM)
                        }
                    }
                }
            }
            return true
        }
    }

    companion object {
        /** A press at least this long before lifting is hold-to-talk. */
        const val HOLD_MS = 350L
        /** Holding a tapped recording this long cancels it. */
        const val CANCEL_MS = 600L
        /** How much bigger the bubble gets while held to talk. */
        const val HOLD_GROWTH = 0.9f
    }
}
