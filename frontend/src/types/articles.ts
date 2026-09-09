export type { Paged as Pagination } from './listings'
export interface Term { id: number; kind: 'category' | 'tag'; slug: string; name: string; active: boolean }
export interface ArticleInput { version: number; slug: string; title: string; summary: string; body: string; media_id: string | null; media_alt: string; term_ids: number[] }
export interface ArticleRevision { id: number; slug: string; title: string; summary: string; body: string; body_html: string; media_id: string | null; media_url: string | null; media_alt: string; terms: Term[] }
export type ArticleStatus = 'draft' | 'published' | 'unpublished_changes'
export interface EditorArticle { id: number; version: number; status: ArticleStatus; public_url: string | null; published_revision_id: number | null; current: ArticleRevision }
export interface ArticleSummary { id: number; version: number; title: string; slug: string; status: ArticleStatus; updated_at: number }
export interface PublicArticle { id: number; slug: string; title: string; summary: string; media_url: string | null; media_alt: string; terms: Term[]; published_at: number; modified_at: number }
export interface PublicArticleDetail extends PublicArticle { body_html: string }
export interface TaxonomyItem { id: number; kind: 'listing_category' | 'category' | 'tag'; slug: string; name: string; active: boolean; version: number }
export interface Seo { title: string; description: string; canonical: string; image: string | null; kind: 'website' | 'article'; structured_data: string }
