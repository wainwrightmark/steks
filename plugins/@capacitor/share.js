import { registerPlugin } from './core.js';

const Share = registerPlugin('Share', {
    web: () => import('../common/web-1431b9bf.js').then(m => new m.ShareWeb()),
});

export { Share };
