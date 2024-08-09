import { writable, readable} from "svelte/store"

export const deviceWidth = writable<number>()

export const supportsWebGL = writable<boolean>(false)

export function checkWebGLSupport() {
  try {
    const canvas = document.createElement("canvas")
    const gl = canvas.getContext("webgl") || canvas.getContext("experimental-webgl")
    supportsWebGL.set(!!gl)
  } catch (e) {
    supportsWebGL.set(false)
  }
}

export const hasKeyboard = readable(false, (set) => {
    const updateKeyboardState = () => set('keyboard' in navigator && navigator.keyboard !== null);

    updateKeyboardState();

    if ('addEventListener' in navigator) {
        navigator.addEventListener('keyboardconnect', updateKeyboardState);
        navigator.addEventListener('keyboarddisconnect', updateKeyboardState);

        return () => {
            navigator.removeEventListener('keyboardconnect', updateKeyboardState);
            navigator.removeEventListener('keyboarddisconnect', updateKeyboardState);
        };
    }
});
