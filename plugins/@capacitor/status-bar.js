import { registerPlugin } from './core.js';

var Style;
(function (Style) {
    /**
     * Light text for dark backgrounds.
     *
     * @since 1.0.0
     */
    Style["Dark"] = "DARK";
    /**
     * Dark text for light backgrounds.
     *
     * @since 1.0.0
     */
    Style["Light"] = "LIGHT";
    /**
     * The style is based on the device appearance.
     * If the device is using Dark mode, the statusbar text will be light.
     * If the device is using Light mode, the statusbar text will be dark.
     * On Android the default will be the one the app was launched with.
     *
     * @since 1.0.0
     */
    Style["Default"] = "DEFAULT";
})(Style || (Style = {}));
var Animation;
(function (Animation) {
    /**
     * No animation during show/hide.
     *
     * @since 1.0.0
     */
    Animation["None"] = "NONE";
    /**
     * Slide animation during show/hide.
     * It doesn't work on iOS 15+.
     *
     * @deprecated Use Animation.Fade or Animation.None instead.
     *
     * @since 1.0.0
     */
    Animation["Slide"] = "SLIDE";
    /**
     * Fade animation during show/hide.
     *
     * @since 1.0.0
     */
    Animation["Fade"] = "FADE";
})(Animation || (Animation = {}));
/**
 * @deprecated Use `Animation`.
 * @since 1.0.0
 */
const StatusBarAnimation = Animation;
/**
 * @deprecated Use `Style`.
 * @since 1.0.0
 */
const StatusBarStyle = Style;

const StatusBar = registerPlugin('StatusBar');

export { Animation, StatusBar, StatusBarAnimation, StatusBarStyle, Style };
