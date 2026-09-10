export interface SeoOverrides { title: string; description: string; image: string; noindex: boolean }
export const emptySeo = (): SeoOverrides => ({ title: '', description: '', image: '', noindex: false })
