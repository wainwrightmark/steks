import { registerPlugin } from './core.js';

const Toast = registerPlugin('Toast', {
    web: () => import('../common/web-7f972d9d.js').then(m => new m.ToastWeb()),
});

export { Toast };
