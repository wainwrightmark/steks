import { registerPlugin } from './core.js';

const Device = registerPlugin('Device', {
    web: () => import('../common/web-37f91109.js').then(m => new m.DeviceWeb()),
});

export { Device };
