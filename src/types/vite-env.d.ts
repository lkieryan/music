/// <reference types="vite/client" />


declare module "virtual:i18next-loader" {
	const resources: typeof import("../../locales/zh-CN/translation.json");
	export default resources;
}
