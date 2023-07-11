import { registerPlugin } from './core.js';

const SplashScreen = registerPlugin('SplashScreen', {
    web: () => import('../common/web-a15fd254.js').then(m => new m.SplashScreenWeb()),
});

export { SplashScreen };
