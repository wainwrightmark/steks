import { registerPlugin } from './core.js';

const App = registerPlugin('App', {
    web: () => import('../common/web-f1011744.js').then(m => new m.AppWeb()),
});

export { App };
