import { registerPlugin } from './core.js';

const Device = registerPlugin('Device', {
    web: () => import('../common/web-ff4b0d2a.js').then(m => new m.DeviceWeb()),
});

export { Device };
