const NS = 'folo-app';
export const getStorageNS = (key: string) => `${NS}:${key}`;
export const clearStorageNS = () => {
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(`${NS}:`)) localStorage.removeItem(k);
  }
};
